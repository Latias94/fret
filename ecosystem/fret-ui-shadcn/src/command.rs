use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_icons::ids;
use fret_runtime::{CommandId, Model};
use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, RowProps, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::cmdk_score;
use fret_ui_kit::headless::cmdk_selection;
use fret_ui_kit::headless::roving_focus;
use fret_ui_kit::primitives::active_descendant as active_desc;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

use crate::layout as shadcn_layout;
use crate::{Dialog, DialogContent, Input, ScrollArea};

fn border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .unwrap_or(theme.colors.panel_border)
}

fn bg(theme: &Theme) -> Color {
    theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or(theme.colors.surface_background)
}

fn item_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(theme.colors.hover_background)
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

pub(crate) fn item_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.command.item.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.command.item.line_height")
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

pub(crate) fn shortcut_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.command.shortcut.text_px")
        .or_else(|| theme.metric_by_key("component.text.sm_px"))
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.command.shortcut.line_height")
        .or_else(|| theme.metric_by_key("component.text.sm_line_height"))
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

/// shadcn/ui `CommandShortcut` (v4).
#[derive(Clone)]
pub struct CommandShortcut {
    text: Arc<str>,
}

impl std::fmt::Debug for CommandShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandShortcut")
            .field("text", &self.text.as_ref())
            .finish()
    }
}

impl CommandShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted-foreground")
            .unwrap_or(theme.colors.text_muted);
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(shortcut_text_style(&theme)),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Clone)]
pub struct Command {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl Command {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children,
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
        let theme = Theme::global(&*cx.app).clone();

        let base = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .merge(ChromeRefinement {
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border(&theme))),
                background: Some(ColorRef::Color(bg(&theme))),
                ..Default::default()
            })
            .merge(self.chrome);

        let props = decl_style::container_props(&theme, base, self.layout);
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Clone)]
pub struct CommandInput {
    model: fret_runtime::Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    disabled: bool,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CommandInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandInput")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .finish()
    }
}

impl CommandInput {
    pub fn new(model: fret_runtime::Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            placeholder: None,
            disabled: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            cx.watch_model(&self.model).observe();

            let border = border(&theme);
            let disabled = self.disabled;
            let mut wrapper = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                self.layout.merge(LayoutRefinement::default().w_full()),
            );
            wrapper.border = Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(1.0),
                left: Px(0.0),
            };
            wrapper.border_color = Some(border);

            let input = Input::new(self.model).a11y_label(
                self.a11y_label
                    .unwrap_or_else(|| Arc::from("Command input")),
            );
            let input = if let Some(placeholder) = self.placeholder.clone() {
                input.placeholder(placeholder)
            } else {
                input
            };

            cx.container(wrapper, move |cx| {
                let mut input = input.into_element(cx);
                if disabled {
                    input = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Generic,
                            disabled: true,
                            ..Default::default()
                        },
                        move |_cx| vec![input],
                    );
                }
                vec![input]
            })
        })
    }
}

#[derive(Clone)]
pub struct CommandItem {
    label: Arc<str>,
    value: Arc<str>,
    disabled: bool,
    keywords: Vec<Arc<str>>,
    checked: bool,
    show_checkmark: bool,
    shortcut: Option<Arc<str>>,
    command: Option<CommandId>,
    on_select: Option<fret_ui::action::OnActivate>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandItem")
            .field("label", &self.label.as_ref())
            .field("value", &self.value.as_ref())
            .field("disabled", &self.disabled)
            .field("keywords_len", &self.keywords.len())
            .field("checked", &self.checked)
            .field("show_checkmark", &self.show_checkmark)
            .field("shortcut", &self.shortcut.as_ref().map(|s| s.as_ref()))
            .field("command", &self.command)
            .field("on_select", &self.on_select.is_some())
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CommandItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            disabled: false,
            keywords: Vec::new(),
            checked: false,
            show_checkmark: false,
            shortcut: None,
            command: None,
            on_select: None,
            children: Vec::new(),
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    /// Additional strings that participate in cmdk-style filtering/ranking.
    ///
    /// This aligns with cmdk's `keywords` prop: matching against these should surface the item even
    /// when the visible label does not contain the query.
    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows a cmdk-style checkmark indicator, with visibility controlled by `checked`.
    pub fn checkmark(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.show_checkmark = true;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn on_select_action(mut self, on_select: fret_ui::action::OnActivate) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}

/// shadcn/ui `CommandGroup` (v4).
#[derive(Clone)]
pub struct CommandGroup {
    heading: Option<Arc<str>>,
    items: Vec<CommandItem>,
}

impl std::fmt::Debug for CommandGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandGroup")
            .field("heading", &self.heading.as_ref().map(|s| s.as_ref()))
            .field("items_len", &self.items.len())
            .finish()
    }
}

impl CommandGroup {
    pub fn new(items: Vec<CommandItem>) -> Self {
        Self {
            heading: None,
            items,
        }
    }

    pub fn heading(mut self, heading: impl Into<Arc<str>>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    pub fn item(mut self, item: CommandItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = CommandItem>) -> Self {
        self.items.extend(items);
        self
    }
}

/// shadcn/ui `CommandSeparator` (v4).
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandSeparator;

impl CommandSeparator {
    pub fn new() -> Self {
        Self
    }
}

/// shadcn/ui `CommandEmpty` (v4).
#[derive(Clone)]
pub struct CommandEmpty {
    text: Arc<str>,
}

impl std::fmt::Debug for CommandEmpty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandEmpty")
            .field("text", &self.text.as_ref())
            .finish()
    }
}

impl CommandEmpty {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.colors.text_muted;
        let text_style = item_text_style(&theme);
        cx.container(ContainerProps::default(), move |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: self.text,
                style: Some(text_style),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        })
    }
}

#[derive(Clone)]
pub enum CommandEntry {
    Item(CommandItem),
    Group(CommandGroup),
    Separator(CommandSeparator),
}

impl From<CommandItem> for CommandEntry {
    fn from(value: CommandItem) -> Self {
        Self::Item(value)
    }
}

impl From<CommandGroup> for CommandEntry {
    fn from(value: CommandGroup) -> Self {
        Self::Group(value)
    }
}

impl From<CommandSeparator> for CommandEntry {
    fn from(value: CommandSeparator) -> Self {
        Self::Separator(value)
    }
}

#[derive(Clone)]
pub struct CommandList {
    items: Vec<CommandItem>,
    disabled: bool,
    empty_text: Arc<str>,
    scroll: LayoutRefinement,
}

impl std::fmt::Debug for CommandList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandList")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("empty_text", &self.empty_text.as_ref())
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl CommandList {
    pub fn new(items: Vec<CommandItem>) -> Self {
        Self {
            items,
            disabled: false,
            empty_text: Arc::from("No results."),
            scroll: LayoutRefinement::default()
                .max_h(MetricRef::Px(Px(300.0)))
                .w_full()
                .min_w_0(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let disabled = self.disabled;
        let items = self.items;

        // Note: `CommandList` is a simple list rendering helper (legacy roving-style semantics).
        // `CommandPalette` is the cmdk-style implementation that keeps focus in the input and
        // drives highlight via `active_descendant` (ADR 0073).
        if items.is_empty() {
            let empty = self.empty_text;
            let fg = theme.colors.text_muted;
            let text_style = item_text_style(&theme);
            return cx.container(ContainerProps::default(), move |cx| {
                vec![cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: empty,
                    style: Some(text_style),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            });
        }

        let disabled_flags: Vec<bool> = items.iter().map(|i| disabled || i.disabled).collect();
        let tab_stop = roving_focus::first_enabled(&disabled_flags);

        let roving = RovingFocusProps {
            enabled: !disabled,
            wrap: true,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        let row_h = MetricRef::space(Space::N8).resolve(&theme);
        let row_gap = MetricRef::space(Space::N2).resolve(&theme);
        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
        let pad_y = MetricRef::space(Space::N1).resolve(&theme);
        let radius = MetricRef::radius(Radius::Sm).resolve(&theme);
        let ring = decl_style::focus_ring(&theme, radius);
        let bg_hover = item_bg_hover(&theme);
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let text_style = item_text_style(&theme);
        let item_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .min_h(MetricRef::Px(row_h))
                .min_w_0(),
        );

        let scroll = self.scroll;

        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::ListBox,
                ..Default::default()
            },
            move |cx| {
                vec![
                    ScrollArea::new(vec![cx.roving_flex(
                        RovingFlexProps {
                            flex: FlexProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.min_height = Some(Px(0.0));
                                    layout
                                },
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                                ..Default::default()
                            },
                            roving,
                        },
                        move |cx| {
                            cx.roving_nav_apg();
                            let mut out = Vec::with_capacity(items.len());

                            for (idx, item) in items.into_iter().enumerate() {
                                let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                                let focusable = tab_stop.is_some_and(|i| i == idx);

                                let label = item.label.clone();
                                let command = item.command;
                                let on_select = item.on_select.clone();
                                let children = item.children;
                                let text_style = text_style.clone();

                                out.push(cx.pressable(
                                    PressableProps {
                                        layout: item_layout,
                                        enabled,
                                        focusable,
                                        focus_ring: Some(ring),
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::ListBoxOption),
                                            label: Some(label.clone()),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, st| {
                                        cx.pressable_dispatch_command_opt(command);
                                        if let Some(on_select) = on_select.clone() {
                                            cx.pressable_add_on_activate(on_select);
                                        }
                                        let hovered = st.hovered && !st.pressed;
                                        let pressed = st.pressed;

                                        let bg = (hovered || pressed).then_some(bg_hover);
                                        let props = ContainerProps {
                                            layout: LayoutStyle::default(),
                                            padding: Edges {
                                                top: pad_y,
                                                right: pad_x,
                                                bottom: pad_y,
                                                left: pad_x,
                                            },
                                            background: bg,
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(radius),
                                        };

                                        vec![cx.container(props, move |cx| {
                                            vec![cx.row(
                                                RowProps {
                                                    layout: LayoutStyle::default(),
                                                    gap: row_gap,
                                                    padding: Edges::all(Px(0.0)),
                                                    justify: MainAlign::SpaceBetween,
                                                    align: CrossAlign::Center,
                                                },
                                                move |cx| {
                                                    if children.is_empty() {
                                                        vec![cx.text_props(TextProps {
                                                            layout: LayoutStyle::default(),
                                                            text: label.clone(),
                                                            style: Some(text_style.clone()),
                                                            color: Some(fg),
                                                            wrap: TextWrap::None,
                                                            overflow: TextOverflow::Clip,
                                                        })]
                                                    } else {
                                                        children
                                                    }
                                                },
                                            )]
                                        })]
                                    },
                                ));
                            }

                            out
                        },
                    )])
                    .refine_layout(scroll)
                    .into_element(cx),
                ]
            },
        )
    }
}

#[derive(Clone)]
pub struct CommandPalette {
    model: Model<String>,
    entries: Vec<CommandEntry>,
    disabled: bool,
    wrap: bool,
    empty_text: Arc<str>,
    a11y_label: Arc<str>,
    placeholder: Option<Arc<str>>,
    input_role: Option<SemanticsRole>,
    input_expanded: Option<bool>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    scroll: LayoutRefinement,
}

#[derive(Clone)]
enum CommandPaletteRenderRow {
    Heading(Arc<str>),
    Separator,
    Item(usize),
}

fn command_palette_render_rows_for_query(
    entries: Vec<CommandEntry>,
    query: &str,
) -> (Vec<CommandPaletteRenderRow>, Vec<CommandItem>) {
    #[derive(Clone)]
    enum PendingRow {
        Heading(Arc<str>),
        Separator,
        Item(CommandItem),
    }

    let score_item = |item: &CommandItem| -> f32 {
        if query.is_empty() {
            return 1.0;
        }

        let mut aliases: Vec<&str> = Vec::with_capacity(1 + item.keywords.len());
        if item.value.as_ref() != item.label.as_ref() {
            aliases.push(item.value.as_ref());
        }
        for kw in &item.keywords {
            aliases.push(kw.as_ref());
        }

        cmdk_score::command_score(item.label.as_ref(), query, &aliases)
    };

    let mut pending_rows: Vec<PendingRow> = Vec::new();
    for entry in entries {
        match entry {
            CommandEntry::Item(item) => {
                let score = score_item(&item);
                if score > 0.0 {
                    pending_rows.push(PendingRow::Item(item));
                }
            }
            CommandEntry::Separator(_) => pending_rows.push(PendingRow::Separator),
            CommandEntry::Group(group) => {
                if group.items.is_empty() {
                    continue;
                }

                if query.is_empty() {
                    if let Some(heading) = group.heading {
                        pending_rows.push(PendingRow::Heading(heading));
                    }
                    pending_rows.extend(group.items.into_iter().map(PendingRow::Item));
                    continue;
                }

                let mut scored: Vec<(usize, f32, CommandItem)> = group
                    .items
                    .into_iter()
                    .enumerate()
                    .filter_map(|(idx, item)| {
                        let score = score_item(&item);
                        (score > 0.0).then_some((idx, score, item))
                    })
                    .collect();

                if scored.is_empty() {
                    continue;
                }

                scored.sort_by(|(a_idx, a_score, _), (b_idx, b_score, _)| {
                    b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
                });
                if let Some(heading) = group.heading {
                    pending_rows.push(PendingRow::Heading(heading));
                }
                pending_rows.extend(
                    scored
                        .into_iter()
                        .map(|(_, _, item)| PendingRow::Item(item)),
                );
            }
        }
    }

    let mut has_item_from: Vec<bool> = vec![false; pending_rows.len() + 1];
    for idx in (0..pending_rows.len()).rev() {
        has_item_from[idx] =
            has_item_from[idx + 1] || matches!(pending_rows[idx], PendingRow::Item(_));
    }

    let mut filtered_rows: Vec<PendingRow> = Vec::with_capacity(pending_rows.len());
    let mut seen_item_before = false;
    let mut prev_is_sep = false;
    for (idx, row) in pending_rows.into_iter().enumerate() {
        match row {
            PendingRow::Separator => {
                if !seen_item_before || !has_item_from[idx + 1] || prev_is_sep {
                    continue;
                }
                prev_is_sep = true;
                filtered_rows.push(PendingRow::Separator);
            }
            PendingRow::Item(item) => {
                seen_item_before = true;
                prev_is_sep = false;
                filtered_rows.push(PendingRow::Item(item));
            }
            PendingRow::Heading(h) => {
                prev_is_sep = false;
                filtered_rows.push(PendingRow::Heading(h));
            }
        }
    }

    let mut items: Vec<CommandItem> = Vec::new();
    let render_rows: Vec<CommandPaletteRenderRow> = filtered_rows
        .into_iter()
        .map(|row| match row {
            PendingRow::Heading(h) => CommandPaletteRenderRow::Heading(h),
            PendingRow::Separator => CommandPaletteRenderRow::Separator,
            PendingRow::Item(item) => {
                let idx = items.len();
                items.push(item);
                CommandPaletteRenderRow::Item(idx)
            }
        })
        .collect();

    (render_rows, items)
}

impl std::fmt::Debug for CommandPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPalette")
            .field("entries_len", &self.entries.len())
            .field("disabled", &self.disabled)
            .field("wrap", &self.wrap)
            .field("empty_text", &self.empty_text.as_ref())
            .field("a11y_label", &self.a11y_label.as_ref())
            .field("input_role", &self.input_role)
            .field("input_expanded", &self.input_expanded)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl CommandPalette {
    pub fn new(model: Model<String>, items: Vec<CommandItem>) -> Self {
        Self {
            model,
            entries: items.into_iter().map(CommandEntry::Item).collect(),
            disabled: false,
            wrap: true,
            empty_text: Arc::from("No results."),
            a11y_label: Arc::from("Command input"),
            placeholder: None,
            input_role: None,
            input_expanded: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            scroll: LayoutRefinement::default()
                .max_h(MetricRef::Px(Px(300.0)))
                .w_full()
                .min_w_0(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn entries(mut self, entries: Vec<CommandEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn empty(mut self, empty: CommandEmpty) -> Self {
        self.empty_text = empty.text;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn input_role(mut self, role: SemanticsRole) -> Self {
        self.input_role = Some(role);
        self
    }

    pub fn input_expanded(mut self, expanded: bool) -> Self {
        self.input_expanded = Some(expanded);
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

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Clone)]
        struct PaletteEntry {
            value: Arc<str>,
            command: Option<CommandId>,
            on_select: Option<fret_ui::action::OnActivate>,
            disabled: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum RowKey {
            Command(CommandId),
            Value(Arc<str>),
        }

        struct KeyHandlerState {
            disabled: Rc<Cell<bool>>,
            wrap: Rc<Cell<bool>>,
            entries: Rc<RefCell<Arc<[PaletteEntry]>>>,
            handler: fret_ui::action::OnKeyDown,
        }

        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let disabled = self.disabled;
            let wrap = self.wrap;
            let query = cx
                .watch_model(&self.model)
                .layout()
                .read_ref(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();

            let (render_rows, items) =
                command_palette_render_rows_for_query(self.entries, query.as_str());

            let items_fingerprint = {
                let mut hasher = DefaultHasher::new();
                query.as_str().hash(&mut hasher);
                render_rows.len().hash(&mut hasher);
                for row in &render_rows {
                    match row {
                        CommandPaletteRenderRow::Heading(h) => {
                            "heading".hash(&mut hasher);
                            h.as_ref().hash(&mut hasher);
                        }
                        CommandPaletteRenderRow::Separator => {
                            "separator".hash(&mut hasher);
                        }
                        CommandPaletteRenderRow::Item(idx) => {
                            "item".hash(&mut hasher);
                            if let Some(item) = items.get(*idx) {
                                item.label.as_ref().hash(&mut hasher);
                                item.value.as_ref().hash(&mut hasher);
                                item.keywords.len().hash(&mut hasher);
                                for kw in &item.keywords {
                                    kw.as_ref().hash(&mut hasher);
                                }
                                item.shortcut
                                    .as_ref()
                                    .map(|s| s.as_ref())
                                    .unwrap_or("")
                                    .hash(&mut hasher);
                                item.disabled.hash(&mut hasher);
                                item.command
                                    .as_ref()
                                    .map(|c| c.as_str())
                                    .unwrap_or("")
                                    .hash(&mut hasher);
                            }
                        }
                    }
                }
                hasher.finish()
            };

            let (entries, disabled_flags): (Vec<PaletteEntry>, Vec<bool>) = items
                .iter()
                .map(|i| {
                    let disabled =
                        disabled || i.disabled || (i.command.is_none() && i.on_select.is_none());
                    (
                        PaletteEntry {
                            value: i.value.clone(),
                            command: i.command.clone(),
                            on_select: i.on_select.clone(),
                            disabled,
                        },
                        disabled,
                    )
                })
                .unzip();
            let entries_arc: Arc<[PaletteEntry]> = Arc::from(entries.into_boxed_slice());

            let active = cx.with_state(CommandPaletteState::default, |st| st.active.clone());
            let active = if let Some(active) = active {
                active
            } else {
                let init = cmdk_selection::clamp_active_index(&disabled_flags, None)
                    .and_then(|i| entries_arc.get(i))
                    .map(|e| e.value.clone());
                let active = cx.app.models_mut().insert(init);
                cx.with_state(CommandPaletteState::default, |st| {
                    st.active = Some(active.clone())
                });
                active
            };

            let _items_changed = cx.with_state(CommandPaletteState::default, |st| {
                if st.items_fingerprint != items_fingerprint {
                    st.items_fingerprint = items_fingerprint;
                    true
                } else {
                    false
                }
            });

            let cur_active = cx.watch_model(&active).cloned().unwrap_or(None);
            let next_active = cur_active
                .as_ref()
                .and_then(|v| {
                    entries_arc
                        .iter()
                        .enumerate()
                        .find(|(idx, e)| {
                            disabled_flags.get(*idx).copied() == Some(false)
                                && e.value.as_ref() == v.as_ref()
                        })
                        .map(|(_, e)| e.value.clone())
                })
                .or_else(|| {
                    entries_arc
                        .iter()
                        .enumerate()
                        .find(|(idx, _)| disabled_flags.get(*idx).copied() == Some(false))
                        .map(|(_, e)| e.value.clone())
                });
            if next_active != cur_active {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&active, |v| *v = next_active.clone());
            }

            let mut row_ids: Vec<fret_ui::elements::GlobalElementId> =
                Vec::with_capacity(items.len());

            let row_h = MetricRef::space(Space::N8).resolve(&theme);
            let row_gap = MetricRef::space(Space::N2).resolve(&theme);
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            let pad_y = MetricRef::space(Space::N1).resolve(&theme);
            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);

            let bg_hover = item_bg_hover(&theme);
            let bg_selected = alpha_mul(bg_hover, 0.85);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or(theme.colors.text_primary);
            let fg_disabled = theme.colors.text_disabled;
            let text_style = item_text_style(&theme);
            let item_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .w_full()
                    .min_h(MetricRef::Px(row_h))
                    .min_w_0(),
            );

            let mut key_counts: HashMap<RowKey, u32> = HashMap::new();

            let active_idx = next_active.as_ref().and_then(|active_value| {
                items.iter().enumerate().find_map(|(idx, item)| {
                    let enabled = disabled_flags.get(idx).copied() == Some(false);
                    if enabled && item.value.as_ref() == active_value.as_ref() {
                        Some(idx)
                    } else {
                        None
                    }
                })
            });
            let item_count = items.len();
            let rows: Vec<AnyElement> = render_rows
                .into_iter()
                .map(|row| match row {
                    CommandPaletteRenderRow::Heading(heading) => {
                        let fg = theme
                            .color_by_key("muted-foreground")
                            .unwrap_or(theme.colors.text_muted);
                        let mut style = shortcut_text_style(&theme);
                        style.weight = FontWeight::MEDIUM;
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout
                                },
                                padding: Edges {
                                    top: Px(8.0),
                                    right: pad_x,
                                    bottom: Px(4.0),
                                    left: pad_x,
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.text_props(TextProps {
                                    layout: LayoutStyle::default(),
                                    text: heading,
                                    style: Some(style),
                                    color: Some(fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                })]
                            },
                        )
                    }
                    CommandPaletteRenderRow::Separator => {
                        let border = border(&theme);
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(Px(1.0));
                                    layout
                                },
                                background: Some(border),
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )
                    }
                    CommandPaletteRenderRow::Item(idx) => {
                        let Some(item) = items.get(idx).cloned() else {
                            return cx.container(ContainerProps::default(), |_cx| Vec::new());
                        };

                        let base = item
                            .command
                            .clone()
                            .map(RowKey::Command)
                            .unwrap_or_else(|| RowKey::Value(item.value.clone()));
                        let count = key_counts.entry(base.clone()).or_insert(0);
                        let occ = *count;
                        *count = count.saturating_add(1);

                        let active_for_row = active.clone();
                        cx.keyed((base, occ), |cx| {
                            let enabled = disabled_flags.get(idx).copied() == Some(false);
                            let selected = active_idx.is_some_and(|i| i == idx);

                            let label = item.label.clone();
                            let value = item.value.clone();
                            let checked = item.checked;
                            let show_checkmark = item.show_checkmark;
                            let shortcut = item.shortcut.clone();
                            let command = item.command;
                            let on_select = item.on_select.clone();
                            let children = item.children;
                            let text_style = text_style.clone();

                            let row = cx.pressable(
                                PressableProps {
                                    layout: item_layout,
                                    enabled,
                                    focusable: false,
                                    focus_ring: None,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::ListBoxOption),
                                        label: Some(label.clone()),
                                        selected,
                                        ..Default::default()
                                    }
                                    .with_collection_position(idx, item_count),
                                    ..Default::default()
                                },
                                move |cx, st| {
                                    cx.pressable_dispatch_command_opt(command);
                                    if let Some(on_select) = on_select.clone() {
                                        cx.pressable_add_on_activate(on_select);
                                    }
                                    if enabled {
                                        let active = active_for_row.clone();
                                        cx.pressable_on_hover_change(Arc::new(
                                            move |host, action_cx, hovered| {
                                                if !hovered {
                                                    return;
                                                }
                                                let current = host
                                                    .models_mut()
                                                    .get_cloned(&active)
                                                    .unwrap_or(None);
                                                let next = Some(value.clone());
                                                if current != next {
                                                    let _ = host
                                                        .models_mut()
                                                        .update(&active, |v| *v = next.clone());
                                                    host.request_redraw(action_cx.window);
                                                }
                                            },
                                        ));
                                    }

                                    let hovered = st.hovered && !st.pressed;
                                    let pressed = st.pressed;
                                    let bg = if selected {
                                        Some(bg_selected)
                                    } else if hovered || pressed {
                                        Some(bg_hover)
                                    } else {
                                        None
                                    };

                                    let props = ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout
                                        },
                                        padding: Edges {
                                            top: pad_y,
                                            right: pad_x,
                                            bottom: pad_y,
                                            left: pad_x,
                                        },
                                        background: bg,
                                        shadow: None,
                                        border: Edges::all(Px(0.0)),
                                        border_color: None,
                                        corner_radii: Corners::all(radius),
                                    };

                                    vec![cx.container(props, move |cx| {
                                        vec![cx.row(
                                            RowProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout
                                                },
                                                gap: row_gap,
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::SpaceBetween,
                                                align: CrossAlign::Center,
                                            },
                                            move |cx| {
                                                if !children.is_empty() {
                                                    return children;
                                                }

                                                let fg = if enabled { fg } else { fg_disabled };

                                                let left = cx.row(
                                                    RowProps {
                                                        layout: LayoutStyle::default(),
                                                        gap: row_gap,
                                                        padding: Edges::all(Px(0.0)),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Center,
                                                    },
                                                    move |cx| {
                                                        let mut out = Vec::with_capacity(2);
                                                        if show_checkmark {
                                                            let icon = decl_icon::icon_with(
                                                                cx,
                                                                ids::ui::CHECK,
                                                                Some(Px(16.0)),
                                                                Some(ColorRef::Color(fg)),
                                                            );
                                                            let icon = cx.opacity(
                                                                if checked { 1.0 } else { 0.0 },
                                                                move |_cx| vec![icon],
                                                            );
                                                            out.push(icon);
                                                        }

                                                        out.push(cx.text_props(TextProps {
                                                            layout: LayoutStyle::default(),
                                                            text: label.clone(),
                                                            style: Some(text_style.clone()),
                                                            color: Some(fg),
                                                            wrap: TextWrap::None,
                                                            overflow: TextOverflow::Clip,
                                                        }));

                                                        out
                                                    },
                                                );

                                                if let Some(shortcut) = shortcut.clone() {
                                                    vec![
                                                        left,
                                                        CommandShortcut::new(shortcut)
                                                            .into_element(cx),
                                                    ]
                                                } else {
                                                    vec![left]
                                                }
                                            },
                                        )]
                                    })]
                                },
                            );

                            row_ids.push(row.id);
                            row
                        })
                    }
                })
                .collect();

            let active_opt = active_desc::active_option_for_index(cx, &row_ids, active_idx);
            let active_descendant = active_opt.map(|opt| opt.node);
            let active_row_element = active_opt.map(|opt| opt.element);

            let border = border(&theme);
            let mut wrapper = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().w_full(),
            );
            wrapper.border = Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(1.0),
                left: Px(0.0),
            };
            wrapper.border_color = Some(border);

            let mut input = Input::new(self.model).a11y_label(self.a11y_label);
            if let Some(placeholder) = self.placeholder {
                input = input.placeholder(placeholder);
            }
            if let Some(role) = self.input_role {
                input = input.a11y_role(role);
            }
            if let Some(active_descendant) = active_descendant {
                input = input.active_descendant(active_descendant);
            }
            if let Some(expanded) = self.input_expanded {
                input = input.expanded(expanded);
            }

            let mut input = input.into_element(cx);

            let key_handler = cx.with_state(
                || {
                    let entries_cell: Rc<RefCell<Arc<[PaletteEntry]>>> =
                        Rc::new(RefCell::new(Arc::from([])));
                    let entries_read = entries_cell.clone();
                    let disabled_cell = Rc::new(Cell::new(false));
                    let wrap_cell = Rc::new(Cell::new(true));

                    let disabled_read = disabled_cell.clone();
                    let wrap_read = wrap_cell.clone();

                    let handler: fret_ui::action::OnKeyDown =
                        Arc::new(move |host, action_cx, down| {
                            if disabled_read.get() {
                                return false;
                            }

                            let entries = entries_read.borrow();
                            let disabled_flags: Vec<bool> =
                                entries.iter().map(|e| e.disabled).collect();

                            match down.key {
                                KeyCode::ArrowDown | KeyCode::ArrowUp => {
                                    let current_value =
                                        host.models_mut().get_cloned(&active).unwrap_or(None);
                                    let current = current_value.as_ref().and_then(|v| {
                                        entries.iter().position(|e| e.value.as_ref() == v.as_ref())
                                    });
                                    let forward = down.key == KeyCode::ArrowDown;
                                    let next_idx = cmdk_selection::next_active_index(
                                        &disabled_flags,
                                        current,
                                        forward,
                                        wrap_read.get(),
                                    );

                                    let next = next_idx
                                        .and_then(|i| entries.get(i))
                                        .map(|e| e.value.clone());
                                    if next != current_value {
                                        let _ = host
                                            .models_mut()
                                            .update(&active, |v| *v = next.clone());
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::Home => {
                                    let current_value =
                                        host.models_mut().get_cloned(&active).unwrap_or(None);
                                    let next_idx = cmdk_selection::first_enabled(&disabled_flags);
                                    let next = next_idx
                                        .and_then(|i| entries.get(i))
                                        .map(|e| e.value.clone());
                                    if next != current_value {
                                        let _ = host
                                            .models_mut()
                                            .update(&active, |v| *v = next.clone());
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::End => {
                                    let current_value =
                                        host.models_mut().get_cloned(&active).unwrap_or(None);
                                    let next_idx = cmdk_selection::last_enabled(&disabled_flags);
                                    let next = next_idx
                                        .and_then(|i| entries.get(i))
                                        .map(|e| e.value.clone());
                                    if next != current_value {
                                        let _ = host
                                            .models_mut()
                                            .update(&active, |v| *v = next.clone());
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::PageDown | KeyCode::PageUp => {
                                    let current_value =
                                        host.models_mut().get_cloned(&active).unwrap_or(None);
                                    let current = current_value.as_ref().and_then(|v| {
                                        entries.iter().position(|e| e.value.as_ref() == v.as_ref())
                                    });
                                    let forward = down.key == KeyCode::PageDown;
                                    let next_idx = cmdk_selection::advance_active_index(
                                        &disabled_flags,
                                        current,
                                        forward,
                                        wrap_read.get(),
                                        10,
                                    );
                                    let next = next_idx
                                        .and_then(|i| entries.get(i))
                                        .map(|e| e.value.clone());
                                    if next != current_value {
                                        let _ = host
                                            .models_mut()
                                            .update(&active, |v| *v = next.clone());
                                        host.request_redraw(action_cx.window);
                                    }
                                    true
                                }
                                KeyCode::Enter | KeyCode::NumpadEnter => {
                                    let current_value =
                                        host.models_mut().get_cloned(&active).unwrap_or(None);
                                    let current = current_value.as_ref().and_then(|v| {
                                        entries.iter().position(|e| e.value.as_ref() == v.as_ref())
                                    });
                                    let Some(idx) = cmdk_selection::clamp_active_index(
                                        &disabled_flags,
                                        current,
                                    ) else {
                                        return false;
                                    };

                                    let Some(entry) = entries.get(idx) else {
                                        return false;
                                    };

                                    if let Some(on_select) = entry.on_select.clone() {
                                        on_select(host, action_cx, ActivateReason::Keyboard);
                                    }

                                    if let Some(command) = entry.command.clone() {
                                        host.dispatch_command(Some(action_cx.window), command);
                                    }
                                    true
                                }
                                _ => false,
                            }
                        });

                    KeyHandlerState {
                        disabled: disabled_cell,
                        wrap: wrap_cell,
                        entries: entries_cell,
                        handler,
                    }
                },
                |state: &mut KeyHandlerState| {
                    state.disabled.set(disabled);
                    state.wrap.set(wrap);
                    *state.entries.borrow_mut() = entries_arc.clone();
                    state.handler.clone()
                },
            );

            cx.key_on_key_down_for(input.id, key_handler);

            if disabled {
                input = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Generic,
                        disabled: true,
                        ..Default::default()
                    },
                    move |_cx| vec![input],
                );
            }

            let list = if row_ids.is_empty() {
                let fg = theme.colors.text_muted;
                let text_style = item_text_style(&theme);
                let empty = self.empty_text;
                cx.container(ContainerProps::default(), move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: empty,
                        style: Some(text_style),
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    })]
                })
            } else {
                let scroll = self.scroll;
                let scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());
                cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::ListBox,
                        ..Default::default()
                    },
                    move |cx| {
                        let scroll_area = ScrollArea::new(vec![cx.flex(
                            FlexProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.min_height = Some(Px(0.0));
                                    layout
                                },
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                                ..Default::default()
                            },
                            move |_cx| rows,
                        )])
                        .scroll_handle(scroll_handle.clone())
                        .refine_layout(scroll)
                        .into_element(cx);

                        if let Some(active_row_element) = active_row_element {
                            let _ = active_desc::scroll_active_element_into_view_y(
                                cx,
                                &scroll_handle,
                                scroll_area.id,
                                active_row_element,
                            );
                        }

                        vec![scroll_area]
                    },
                )
            };

            Command::new(vec![cx.container(wrapper, move |_cx| vec![input]), list])
                .refine_style(self.chrome)
                .refine_layout(self.layout)
                .into_element(cx)
        })
    }
}

#[derive(Clone)]
pub struct CommandDialog {
    open: Model<bool>,
    query: Model<String>,
    entries: Vec<CommandEntry>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    wrap: bool,
    empty_text: Arc<str>,
}

impl std::fmt::Debug for CommandDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDialog")
            .field("open", &"<model>")
            .field("query", &"<model>")
            .field("entries_len", &self.entries.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("disabled", &self.disabled)
            .field("wrap", &self.wrap)
            .field("empty_text", &self.empty_text.as_ref())
            .finish()
    }
}

impl CommandDialog {
    pub fn new(open: Model<bool>, query: Model<String>, items: Vec<CommandItem>) -> Self {
        Self {
            open,
            query,
            entries: items.into_iter().map(CommandEntry::Item).collect(),
            a11y_label: None,
            disabled: false,
            wrap: true,
            empty_text: Arc::from("No results."),
        }
    }

    pub fn entries(mut self, entries: Vec<CommandEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn empty(mut self, empty: CommandEmpty) -> Self {
        self.empty_text = empty.text;
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = self.open;
        let query = self.query;
        let entries = self.entries;
        let a11y_label = self.a11y_label;
        let disabled = self.disabled;
        let wrap = self.wrap;
        let empty_text = self.empty_text;

        Dialog::new(open).into_element(cx, trigger, move |cx| {
            let palette = CommandPalette::new(query, Vec::new())
                .entries(entries)
                .a11y_label(a11y_label.unwrap_or_else(|| Arc::from("Command palette")))
                .disabled(disabled)
                .wrap(wrap)
                .empty_text(empty_text)
                .into_element(cx);

            DialogContent::new(vec![palette])
                .refine_style(ChromeRefinement::default().p(Space::N0))
                .into_element(cx)
        })
    }
}

#[derive(Default)]
struct CommandPaletteState {
    active: Option<Model<Option<Arc<str>>>>,
    items_fingerprint: u64,
}

pub fn command<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    Command::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, SemanticsRole, Size, SvgId,
        SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<String>,
        items: Vec<CommandItem>,
    ) -> fret_core::NodeId {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "cmdk", |cx| {
                vec![CommandPalette::new(model, items).into_element(cx)]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn row_signatures(rows: &[CommandPaletteRenderRow], items: &[CommandItem]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                CommandPaletteRenderRow::Heading(h) => format!("H:{h}"),
                CommandPaletteRenderRow::Separator => "S".to_string(),
                CommandPaletteRenderRow::Item(idx) => {
                    let label = items
                        .get(*idx)
                        .map(|i| i.label.as_ref())
                        .unwrap_or("<missing>");
                    format!("I:{label}")
                }
            })
            .collect()
    }

    #[test]
    fn cmdk_arrow_moves_highlight_while_focus_stays_in_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        // Move highlight down.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(active_node.pos_in_set, Some(2));
        assert_eq!(active_node.set_size, Some(2));
        assert!(
            active_node.flags.selected,
            "highlighted row should be selected"
        );
    }

    #[test]
    fn cmdk_scrolls_active_option_into_view_while_focus_stays_in_input() {
        fn rects_intersect(a: Rect, b: Rect) -> bool {
            let ax1 = a.origin.x.0;
            let ay1 = a.origin.y.0;
            let ax2 = ax1 + a.size.width.0;
            let ay2 = ay1 + a.size.height.0;

            let bx1 = b.origin.x.0;
            let by1 = b.origin.y.0;
            let bx2 = bx1 + b.size.width.0;
            let by2 = by1 + b.size.height.0;

            ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1
        }

        fn find_scroll_viewport(ui: &UiTree<App>, node: fret_core::NodeId, target_h: Px) -> Rect {
            let path = ui.debug_node_path(node);
            for &ancestor in path.iter().rev() {
                let Some(bounds) = ui.debug_node_bounds(ancestor) else {
                    continue;
                };
                if (bounds.size.height.0 - target_h.0).abs() <= 0.5 {
                    return bounds;
                }
            }
            panic!("expected a scroll viewport ancestor with height ~= {target_h:?}");
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<String>,
            items: Vec<CommandItem>,
        ) -> fret_core::NodeId {
            let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            fret_ui_kit::OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "cmdk",
                |cx| {
                    vec![
                        CommandPalette::new(model, items)
                            .refine_scroll_layout(
                                LayoutRefinement::default()
                                    .h_px(MetricRef::Px(Px(40.0)))
                                    .max_h(MetricRef::Px(Px(40.0))),
                            )
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<CommandItem> = (0..12)
            .map(|i| {
                CommandItem::new(format!("Item {i}")).on_select(CommandId::new(format!("i{i}")))
            })
            .collect();

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        // Move highlight down to the end of the list without moving focus away from the input.
        for _ in 0..11 {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::KeyDown {
                    key: KeyCode::ArrowDown,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");

        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Item 11"));

        let viewport = find_scroll_viewport(&ui, active, Px(40.0));
        assert!(
            rects_intersect(viewport, active_node.bounds),
            "expected active option to intersect the scroll viewport after scroll-into-view"
        );
    }

    #[test]
    fn cmdk_hover_moves_highlight_while_focus_stays_in_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
            CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_bounds = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .map(|n| n.bounds)
            .expect("Beta row bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(
                    Px(beta_bounds.origin.x.0 + 1.0),
                    Px(beta_bounds.origin.y.0 + 1.0),
                ),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(active_node.pos_in_set, Some(2));
        assert_eq!(active_node.set_size, Some(3));
    }

    #[test]
    fn cmdk_highlight_tracks_value_across_reorder() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
            CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items.clone(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        // Highlight "Gamma".
        for _ in 0..2 {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::KeyDown {
                    key: KeyCode::ArrowDown,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
        }

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            items,
        );

        // Reorder items and ensure highlight stays on the same value (not the same index).
        let reordered = vec![
            CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
            CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
            CommandItem::new("Beta").on_select(CommandId::new("beta")),
        ];
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            reordered,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        assert_eq!(focus, input, "focus should remain on the input node");

        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TextField && n.id == focus)
            .expect("focused text field node");
        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Gamma"));
    }

    #[test]
    fn cmdk_filters_by_value_alias() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::from("settings"));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Open")
                .value("settings")
                .on_select(CommandId::new("open")),
            CommandItem::new("Close")
                .value("close")
                .on_select(CommandId::new("close")),
        ];

        let _root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let options: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .collect();
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].label.as_deref(), Some("Open"));
    }

    #[test]
    fn command_palette_groups_flatten_into_headings_separators_and_items() {
        let entries = vec![
            CommandGroup::new(vec![CommandItem::new("Alpha"), CommandItem::new("Beta")])
                .heading("Basics")
                .into(),
            CommandSeparator::new().into(),
            CommandItem::new("Gamma").into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "");
        assert_eq!(items.len(), 3);
        assert_eq!(
            row_signatures(&rows, &items),
            vec![
                "H:Basics".to_string(),
                "I:Alpha".to_string(),
                "I:Beta".to_string(),
                "S".to_string(),
                "I:Gamma".to_string()
            ]
        );
    }

    #[test]
    fn command_palette_filter_hides_empty_groups_and_trims_separators() {
        let entries = vec![
            CommandGroup::new(vec![CommandItem::new("Alpha"), CommandItem::new("Beta")])
                .heading("Basics")
                .into(),
            CommandSeparator::new().into(),
            CommandGroup::new(vec![CommandItem::new("Gamma")])
                .heading("Advanced")
                .into(),
            CommandSeparator::new().into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "gam");
        assert_eq!(items.len(), 1);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["H:Advanced".to_string(), "I:Gamma".to_string()]
        );
    }

    #[test]
    fn command_palette_collapses_consecutive_separators() {
        let entries = vec![
            CommandItem::new("Alpha").into(),
            CommandSeparator::new().into(),
            CommandSeparator::new().into(),
            CommandItem::new("Beta").into(),
            CommandSeparator::new().into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "");
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["I:Alpha".to_string(), "S".to_string(), "I:Beta".to_string()]
        );
    }

    #[test]
    fn cmdk_filters_by_keywords() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::from("prefs"));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            CommandItem::new("Open")
                .keywords(["preferences", "prefs"])
                .on_select(CommandId::new("open")),
            CommandItem::new("Close").on_select(CommandId::new("close")),
        ];

        let _root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            items,
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let options: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .collect();
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].label.as_deref(), Some("Open"));
    }
}
