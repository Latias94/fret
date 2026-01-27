use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, KeyCode, NodeId, Px, SemanticsRole, TextStyle,
};
use fret_icons::ids;
use fret_runtime::WindowCommandGatingService;
use fret_runtime::WindowCommandGatingSnapshot;
use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, KeymapService, Platform, PlatformCapabilities,
    format_sequence,
};
use fret_runtime::{CommandMeta, Model};
use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, RowProps, SizeStyle,
    TextInputProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_headless::cmdk_score;
use fret_ui_headless::cmdk_selection;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::active_descendant as active_desc;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::layout as shadcn_layout;
use crate::{Dialog, DialogContent, ScrollArea};

#[derive(Debug, Clone, Copy, Default)]
pub struct CommandCatalogOptions {
    /// When `true`, commands that fail their `when` gating are excluded from the palette instead of
    /// being rendered as disabled rows.
    pub hide_disabled: bool,
}

pub fn command_palette_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext {
        platform: Platform::current(),
        caps,
        // Best-effort: the command palette itself is typically presented in a modal dialog.
        ui_has_modal: true,
        window_arbitration: None,
        // Best-effort: treat the palette as a global discovery surface, not a text-editing scope.
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        dispatch_phase: InputDispatchPhase::Bubble,
    }
}

fn command_item_from_meta_with_gating<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    gating: &WindowCommandGatingSnapshot,
    id: &CommandId,
    meta: &CommandMeta,
) -> CommandItem {
    let input_ctx = gating.input_ctx();

    let mut keywords: Vec<Arc<str>> = meta.keywords.clone();
    keywords.push(Arc::from(id.as_str()));
    if let Some(category) = meta.category.as_ref() {
        keywords.push(category.clone());
    }
    if let Some(description) = meta.description.as_ref() {
        keywords.push(description.clone());
    }

    let shortcut: Option<Arc<str>> = cx
        .app
        .global::<KeymapService>()
        .and_then(|svc| {
            svc.keymap
                .display_shortcut_for_command_sequence(input_ctx, id)
        })
        .map(|seq| Arc::from(format_sequence(input_ctx.platform, &seq)));

    let disabled = !gating.is_enabled_for_command(id, meta);

    let mut item = CommandItem::new(meta.title.clone())
        .value(Arc::from(id.as_str()))
        .keywords(keywords)
        .disabled(disabled)
        .on_select(id.clone());
    if let Some(shortcut) = shortcut {
        item = item.shortcut(shortcut);
    }
    item
}

pub fn command_entries_from_host_commands<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Vec<CommandEntry> {
    command_entries_from_host_commands_with_options(cx, CommandCatalogOptions::default())
}

pub fn command_entries_from_host_commands_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
) -> Vec<CommandEntry> {
    let fallback_input_ctx = command_palette_input_context(&*cx.app);
    let snapshot = cx
        .app
        .global::<WindowCommandGatingService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .cloned()
        .unwrap_or_else(|| {
            fret_runtime::snapshot_for_window_with_input_ctx_fallback(
                &*cx.app,
                cx.window,
                fallback_input_ctx,
            )
        });

    // Best-effort: treat the command palette as a global discovery surface, even when the window
    // input context reflects focus in the palette input itself.
    let mut input_ctx = snapshot.input_ctx().clone();
    input_ctx.ui_has_modal = true;
    input_ctx.focus_is_text_input = false;
    input_ctx.dispatch_phase = InputDispatchPhase::Bubble;

    let gating = snapshot.with_input_ctx(input_ctx);
    command_entries_from_host_commands_with_gating_snapshot(cx, options, &gating)
}

pub fn command_entries_from_host_commands_with_gating_snapshot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
    gating: &WindowCommandGatingSnapshot,
) -> Vec<CommandEntry> {
    let mut commands: Vec<(CommandId, CommandMeta)> = cx
        .app
        .commands()
        .iter()
        .filter_map(|(id, meta)| (!meta.hidden).then_some((id.clone(), meta.clone())))
        .collect();

    commands.sort_by(|(a_id, a_meta), (b_id, b_meta)| {
        match (&a_meta.category, &b_meta.category) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.as_ref().cmp(b.as_ref()),
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a_meta.title.as_ref().cmp(b_meta.title.as_ref()))
        .then_with(|| a_id.as_str().cmp(b_id.as_str()))
    });

    let mut root_items: Vec<CommandItem> = Vec::new();
    let mut groups: std::collections::BTreeMap<Arc<str>, Vec<CommandItem>> =
        std::collections::BTreeMap::new();

    for (id, meta) in &commands {
        let disabled = !gating.is_enabled_for_command(id, meta);
        if disabled && options.hide_disabled {
            continue;
        }

        let item = command_item_from_meta_with_gating(cx, gating, id, meta);

        if let Some(category) = meta.category.clone() {
            groups.entry(category).or_default().push(item);
        } else {
            root_items.push(item);
        }
    }

    let mut entries: Vec<CommandEntry> = Vec::new();
    entries.extend(root_items.into_iter().map(CommandEntry::Item));
    entries.extend(
        groups.into_iter().map(|(category, items)| {
            CommandEntry::Group(CommandGroup::new(items).heading(category))
        }),
    );
    entries
}

fn border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .expect("missing theme token: border/input")
}

fn bg(theme: &Theme) -> Color {
    theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("background"))
        .expect("missing theme token: popover/background")
}

fn item_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("muted"))
        .expect("missing theme token: accent/muted")
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn command_text_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Arc<str>,
    placeholder: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    height: Px,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let fg = theme.color_required("foreground");
    let placeholder_fg = theme.color_required("muted-foreground");
    let pad_y = MetricRef::space(Space::N3).resolve(&theme);

    let mut chrome = TextInputStyle::from_theme(theme.snapshot());
    // shadcn/ui v4: cmdk input uses `py-3` and relies on the wrapper for horizontal padding.
    chrome.padding = Edges {
        top: pad_y,
        right: Px(0.0),
        bottom: pad_y,
        left: Px(0.0),
    };
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.border = Edges::all(Px(0.0));
    chrome.background = Color::TRANSPARENT;
    chrome.border_color = Color::TRANSPARENT;
    chrome.border_color_focused = Color::TRANSPARENT;
    chrome.focus_ring = None;
    chrome.text_color = fg;
    chrome.placeholder_color = placeholder_fg;
    chrome.caret_color = fg;

    let mut props = TextInputProps::new(model);
    props.a11y_label = Some(a11y_label);
    props.a11y_role = a11y_role;
    props.placeholder = placeholder;
    props.active_descendant = active_descendant;
    props.expanded = expanded;
    props.chrome = chrome;
    props.text_style = item_text_style(&theme);
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Px(height),
        min_width: Some(Px(0.0)),
        min_height: Some(Px(0.0)),
        ..Default::default()
    };
    props.layout.overflow = Overflow::Clip;

    cx.text_input(props)
}

fn cmdk_highlighted_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    query: &str,
    fg: Color,
    text_style: TextStyle,
) -> AnyElement {
    let text_px = text_style.size;
    let text_weight = text_style.weight;
    let text_line_height = text_style.line_height;
    let text_letter_spacing_em = text_style.letter_spacing_em;

    let apply_text_style = |mut text: fret_ui_kit::UiBuilder<fret_ui_kit::ui::TextBox>| {
        text = text.text_size_px(text_px).font_weight(text_weight).nowrap();

        if let Some(line_height) = text_line_height {
            text = text.line_height_px(line_height);
        }

        if let Some(letter_spacing_em) = text_letter_spacing_em {
            text = text.letter_spacing_em(letter_spacing_em);
        }

        text
    };

    if query.is_empty() {
        return apply_text_style(ui::text(cx, label))
            .layout(LayoutRefinement::default().w_full().min_w_0().flex_1())
            .text_color(ColorRef::Color(fg))
            .into_element(cx);
    }

    let theme = Theme::global(&*cx.app).clone();
    let muted_fg = theme.color_required("muted-foreground");

    let ranges = cmdk_score::command_match_ranges(label.as_ref(), query);
    if ranges.is_empty() {
        return apply_text_style(ui::text(cx, label))
            .layout(LayoutRefinement::default().w_full().min_w_0().flex_1())
            .text_color(ColorRef::Color(fg))
            .into_element(cx);
    }

    let label_chars: Vec<char> = label.chars().collect();
    let mut pieces: Vec<AnyElement> = Vec::new();

    let mut cursor: usize = 0;
    for range in ranges {
        let start = range.start.min(label_chars.len());
        let end = range.end.min(label_chars.len());
        if cursor < start {
            let text: Arc<str> = label_chars[cursor..start]
                .iter()
                .copied()
                .collect::<String>()
                .into();
            pieces.push(
                apply_text_style(ui::text(cx, text))
                    .text_color(ColorRef::Color(muted_fg))
                    .into_element(cx),
            );
        }
        if start < end {
            let text: Arc<str> = label_chars[start..end]
                .iter()
                .copied()
                .collect::<String>()
                .into();
            pieces.push(
                apply_text_style(ui::text(cx, text))
                    .text_color(ColorRef::Color(fg))
                    .into_element(cx),
            );
        }
        cursor = end;
    }

    if cursor < label_chars.len() {
        let text: Arc<str> = label_chars[cursor..]
            .iter()
            .copied()
            .collect::<String>()
            .into();
        pieces.push(
            apply_text_style(ui::text(cx, text))
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx),
        );
    }

    cx.row(
        RowProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.min_width = Some(Px(0.0));
                layout.flex.grow = 1.0;
                layout.flex.shrink = 1.0;
                layout.flex.basis = Length::Px(Px(0.0));
                layout
            },
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
        },
        move |_cx| pieces,
    )
}

pub(crate) fn item_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.command.item.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.command.item.line_height")
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

fn heading_text_style(theme: &Theme) -> TextStyle {
    // shadcn/ui v4: command group headings use `text-xs` / `leading-4`.
    let size = theme
        .metric_by_key("component.command.heading.text_px")
        .unwrap_or(Px(12.0));
    let line_height = theme
        .metric_by_key("component.command.heading.line_height")
        .unwrap_or(Px(16.0));

    TextStyle {
        font: FontId::default(),
        size,
        weight: FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

pub(crate) fn shortcut_text_style(theme: &Theme) -> TextStyle {
    let base_size = theme.metric_required("font.size");
    let base_line_height = theme.metric_required("font.line_height");

    let px = theme
        .metric_by_key("component.command.shortcut.text_px")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
        .unwrap_or_else(|| Px((base_size.0 - 2.0).max(10.0)));
    let line_height = theme
        .metric_by_key("component.command.shortcut.line_height")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT))
        .unwrap_or_else(|| Px((base_line_height.0 - 4.0).max(px.0)));
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        // new-york-v4: `tracking-widest`.
        letter_spacing_em: Some(0.10),
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
        let fg = theme.color_required("muted-foreground");
        let style = shortcut_text_style(&theme);
        let mut text = ui::text(cx, self.text)
            .layout(LayoutRefinement::default().flex_shrink_0().ml_auto())
            .text_size_px(style.size)
            .font_weight(style.weight)
            .nowrap()
            .text_color(ColorRef::Color(fg));

        if let Some(line_height) = style.line_height {
            text = text.line_height_px(line_height);
        }

        if let Some(letter_spacing_em) = style.letter_spacing_em {
            text = text.letter_spacing_em(letter_spacing_em);
        }

        text.into_element(cx)
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
    chrome: ChromeRefinement,
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
            chrome: ChromeRefinement::default(),
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
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
            let wrapper_h = theme
                .metric_by_key("component.command.input.wrapper_height")
                .unwrap_or(Px(36.0));
            let input_h = theme
                .metric_by_key("component.command.input.height")
                .unwrap_or(Px(40.0));
            let icon_size = theme
                .metric_by_key("component.command.input.icon_size")
                .unwrap_or(Px(16.0));
            let pad_x = theme
                .metric_by_key("component.command.input.padding_x")
                .unwrap_or(Px(12.0));
            let gap = theme
                .metric_by_key("component.command.input.gap")
                .unwrap_or(Px(8.0));

            let chrome = self.chrome.clone();
            let border_w = chrome
                .border_width
                .as_ref()
                .map(|m| m.resolve(&theme))
                .unwrap_or(Px(1.0));
            let border_color = chrome
                .border_color
                .as_ref()
                .map(|c| c.resolve(&theme))
                .unwrap_or(border);
            let background = chrome.background.as_ref().map(|c| c.resolve(&theme));
            let radius = chrome
                .radius
                .as_ref()
                .map(|m| m.resolve(&theme))
                .unwrap_or(Px(0.0));

            let padding = chrome.padding.clone().unwrap_or_default();
            let pad_top = padding.top.map(|m| m.resolve(&theme)).unwrap_or(Px(0.0));
            let pad_right = padding.right.map(|m| m.resolve(&theme)).unwrap_or(pad_x);
            let pad_bottom = padding.bottom.map(|m| m.resolve(&theme)).unwrap_or(Px(0.0));
            let pad_left = padding.left.map(|m| m.resolve(&theme)).unwrap_or(pad_x);

            let mut wrapper = decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                self.layout.merge(LayoutRefinement::default().w_full()),
            );
            wrapper.border = Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: border_w,
                left: Px(0.0),
            };
            wrapper.border_color = Some(border_color);
            wrapper.background = background;
            wrapper.corner_radii = Corners::all(radius);
            if matches!(wrapper.layout.size.height, Length::Auto) {
                wrapper.layout.size.height = Length::Px(wrapper_h);
            }
            wrapper.padding = Edges {
                top: pad_top,
                right: pad_right,
                bottom: pad_bottom,
                left: pad_left,
            };

            cx.container(wrapper, move |cx| {
                let a11y_label = self
                    .a11y_label
                    .clone()
                    .unwrap_or_else(|| Arc::from("Command input"));
                let placeholder = self.placeholder.clone();
                let icon_fg = theme.color_required("muted-foreground");

                let icon = decl_icon::icon_with(
                    cx,
                    ids::ui::SEARCH,
                    Some(icon_size),
                    Some(ColorRef::Color(icon_fg)),
                );
                let icon = cx.opacity(0.5, move |_cx| vec![icon]);

                let input = command_text_input(
                    cx,
                    self.model.clone(),
                    a11y_label,
                    placeholder,
                    Some(SemanticsRole::ComboBox),
                    None,
                    None,
                    input_h,
                );

                let mut row = cx.row(
                    RowProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                    },
                    move |_cx| vec![icon, input],
                );

                if disabled {
                    row = cx.opacity(0.5, move |_cx| vec![row]);
                }

                let mut input = row;
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
        let fg = theme.color_required("muted-foreground");
        let text_style = item_text_style(&theme);
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                // new-york-v4: `py-6 text-center text-sm`.
                padding: Edges {
                    top: Px(24.0),
                    right: Px(0.0),
                    bottom: Px(24.0),
                    left: Px(0.0),
                },
                ..Default::default()
            },
            move |cx| {
                vec![cx.row(
                    RowProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                    },
                    move |cx| {
                        let mut text = ui::text(cx, self.text)
                            .text_size_px(text_style.size)
                            .font_weight(text_style.weight)
                            .nowrap()
                            .text_color(ColorRef::Color(fg));

                        if let Some(line_height) = text_style.line_height {
                            text = text.line_height_px(line_height);
                        }

                        if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                            text = text.letter_spacing_em(letter_spacing_em);
                        }

                        vec![text.into_element(cx)]
                    },
                )]
            },
        )
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
    highlight_query: Option<Model<String>>,
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
            highlight_query: None,
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

    /// Provide an optional query model to render cmdk-style match highlighting for default rows.
    ///
    /// This is intended for legacy/roving lists that want cmdk-like visuals without adopting the
    /// full `CommandPalette` surface.
    pub fn highlight_query_model(mut self, model: Model<String>) -> Self {
        self.highlight_query = Some(model);
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let disabled = self.disabled;
            let items = self.items;
            let highlight_query = self.highlight_query;

            // Note: `CommandList` is a simple list rendering helper (legacy roving-style semantics).
            // `CommandPalette` is the cmdk-style implementation that keeps focus in the input and
            // drives highlight via `active_descendant` (ADR 0073).
            if items.is_empty() {
                let empty = self.empty_text;
                return CommandEmpty::new(empty).into_element(cx);
            }

            let query_for_render: Arc<str> = highlight_query
                .as_ref()
                .and_then(|model| {
                    cx.watch_model(model)
                        .layout()
                        .read_ref(|s| s.trim().to_ascii_lowercase())
                        .ok()
                })
                .and_then(|trimmed| (!trimmed.is_empty()).then(|| Arc::<str>::from(trimmed)))
                .unwrap_or_else(|| Arc::from(""));

            let disabled_flags: Vec<bool> = items.iter().map(|i| disabled || i.disabled).collect();
            let tab_stop = roving_focus_group::first_enabled(&disabled_flags);

            let roving = RovingFocusProps {
                enabled: !disabled,
                wrap: true,
                disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
                ..Default::default()
            };

            let row_h = MetricRef::space(Space::N8).resolve(&theme);
            let row_gap = MetricRef::space(Space::N2).resolve(&theme);
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            // new-york-v4: `py-1.5` for `CommandItem` in the base `Command` surface.
            let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);
            let ring = decl_style::focus_ring(&theme, radius);
            let bg_hover = item_bg_hover(&theme);
            let fg = theme.color_required("foreground");
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
                                    // new-york-v4: `CommandList` uses `scroll-py-1` and is typically
                                    // wrapped in `CommandGroup` which uses `p-1`.
                                    padding: Edges::all(Px(4.0)),
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

                                    let query_for_row = query_for_render.clone();
                                    let value_key = item.value.clone();
                                    let label = item.label.clone();
                                    let command = item.command;
                                    let on_select = item.on_select.clone();
                                    let children = item.children;
                                    let text_style = text_style.clone();

                                    out.push(cx.keyed(value_key, |cx| {
                                        cx.pressable(
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
                                                cx.pressable_dispatch_command_if_enabled_opt(
                                                    command,
                                                );
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
                                                    ..Default::default()
                                                };

                                                vec![cx.container(props, move |cx| {
                                                    vec![cx.row(
                                                        RowProps {
                                                            layout: LayoutStyle::default(),
                                                            gap: row_gap,
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Center,
                                                        },
                                                        move |cx| {
                                                            if children.is_empty() {
                                                                vec![cmdk_highlighted_label(
                                                                    cx,
                                                                    label.clone(),
                                                                    query_for_row.as_ref(),
                                                                    fg,
                                                                    text_style.clone(),
                                                                )]
                                                            } else {
                                                                children
                                                            }
                                                        },
                                                    )]
                                                })]
                                            },
                                        )
                                    }));
                                }

                                out
                            },
                        )])
                        .refine_layout(scroll)
                        .into_element(cx),
                    ]
                },
            )
        })
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
    input_wrapper_h: MetricRef,
    input_h: MetricRef,
    input_icon_size: MetricRef,
    item_pad_y: MetricRef,
    group_pad_x: MetricRef,
    group_pad_y: MetricRef,
    group_next_top_pad_zero: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    scroll: LayoutRefinement,
}

#[derive(Clone)]
enum CommandPaletteRenderRow {
    Heading(Arc<str>),
    GroupPad,
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
    if query.is_empty() {
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

                    if let Some(heading) = group.heading {
                        pending_rows.push(PendingRow::Heading(heading));
                    }
                    pending_rows.extend(group.items.into_iter().map(PendingRow::Item));
                }
            }
        }
    } else {
        // cmdk sorts items globally by score and groups by highest item score. It also pushes
        // grouped items below ungrouped items.
        let mut root_items: Vec<(usize, f32, CommandItem)> = Vec::new();
        let mut groups: Vec<(usize, f32, Option<Arc<str>>, Vec<(usize, f32, CommandItem)>)> =
            Vec::new();

        for (entry_idx, entry) in entries.into_iter().enumerate() {
            match entry {
                CommandEntry::Item(item) => {
                    let score = score_item(&item);
                    if score > 0.0 {
                        root_items.push((entry_idx, score, item));
                    }
                }
                CommandEntry::Separator(_) => {}
                CommandEntry::Group(group) => {
                    if group.items.is_empty() {
                        continue;
                    }

                    let mut scored_items: Vec<(usize, f32, CommandItem)> = group
                        .items
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, item)| {
                            let score = score_item(&item);
                            (score > 0.0).then_some((idx, score, item))
                        })
                        .collect();

                    if scored_items.is_empty() {
                        continue;
                    }

                    scored_items.sort_by(|(a_idx, a_score, _), (b_idx, b_score, _)| {
                        b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
                    });

                    let max_score = scored_items
                        .iter()
                        .map(|(_, score, _)| *score)
                        .fold(0.0_f32, f32::max);

                    groups.push((entry_idx, max_score, group.heading, scored_items));
                }
            }
        }

        root_items.sort_by(|(a_idx, a_score, _), (b_idx, b_score, _)| {
            b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
        });
        groups.sort_by(|(a_idx, a_score, _, _), (b_idx, b_score, _, _)| {
            b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
        });

        pending_rows.extend(
            root_items
                .into_iter()
                .map(|(_, _, item)| PendingRow::Item(item)),
        );
        for (_, _, heading, scored_items) in groups {
            if let Some(heading) = heading {
                pending_rows.push(PendingRow::Heading(heading));
            }
            pending_rows.extend(
                scored_items
                    .into_iter()
                    .map(|(_, _, item)| PendingRow::Item(item)),
            );
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
    let mut saw_heading = false;
    let render_rows: Vec<CommandPaletteRenderRow> = filtered_rows
        .into_iter()
        .flat_map(|row| match row {
            PendingRow::Heading(h) => {
                let mut out = Vec::new();
                if saw_heading {
                    out.push(CommandPaletteRenderRow::GroupPad);
                }
                out.push(CommandPaletteRenderRow::Heading(h));
                saw_heading = true;
                out
            }
            PendingRow::Separator => vec![CommandPaletteRenderRow::Separator],
            PendingRow::Item(item) => {
                let idx = items.len();
                items.push(item);
                vec![CommandPaletteRenderRow::Item(idx)]
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
            input_role: Some(SemanticsRole::ComboBox),
            input_expanded: None,
            input_wrapper_h: MetricRef::Px(Px(36.0)),
            input_h: MetricRef::Px(Px(40.0)),
            input_icon_size: MetricRef::Px(Px(16.0)),
            item_pad_y: MetricRef::space(Space::N1p5),
            group_pad_x: MetricRef::space(Space::N1),
            group_pad_y: MetricRef::space(Space::N1),
            group_next_top_pad_zero: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            scroll: LayoutRefinement::default()
                .max_h(MetricRef::Px(Px(300.0)))
                .w_full()
                .min_w_0(),
        }
    }

    /// Creates a command palette with a controlled/uncontrolled query model (cmdk-style input).
    ///
    /// Upstream cmdk/shadcn surfaces typically own this state internally, but exposing a Radix-like
    /// `value` / `defaultValue` style entry point makes it easier to compose dialogs and recipes.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        query: Option<Model<String>>,
        default_query: String,
        items: Vec<CommandItem>,
    ) -> Self {
        let query = controllable_state::use_controllable_model(cx, query, || default_query).model();
        Self::new(query, items)
    }

    /// Applies the shadcn/ui v4 `CommandDialog`-specific overrides used by the new-york theme.
    ///
    /// On the web these come from selector-based rules on the command root:
    /// - `**:data-[slot=command-input-wrapper]:h-12`
    /// - `[&_[cmdk-input-wrapper]_svg]:h-5` / `w-5`
    /// - `[&_[cmdk-item]]:py-3`
    /// - `[&_[cmdk-group]]:px-2`
    pub fn command_dialog_defaults(mut self) -> Self {
        self.input_wrapper_h = MetricRef::Px(Px(48.0));
        self.input_h = MetricRef::Px(Px(48.0));
        self.input_icon_size = MetricRef::Px(Px(20.0));
        self.item_pad_y = MetricRef::space(Space::N3);
        self.group_pad_x = MetricRef::space(Space::N2);
        self.group_pad_y = MetricRef::space(Space::N1);
        self.group_next_top_pad_zero = true;
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
            let input_wrapper_h_fallback = self.input_wrapper_h.resolve(&theme);
            let input_h_fallback = self.input_h.resolve(&theme);
            let input_icon_size_fallback = self.input_icon_size.resolve(&theme);
            let item_pad_y = self.item_pad_y.resolve(&theme);
            let group_pad_x = self.group_pad_x.resolve(&theme);
            let group_pad_y = self.group_pad_y.resolve(&theme);
            let group_next_top_pad_zero = self.group_next_top_pad_zero;

            let disabled = self.disabled;
            let wrap = self.wrap;
            let query = cx
                .watch_model(&self.model)
                .layout()
                .read_ref(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();
            let query_for_render: Arc<str> = Arc::from(query.as_str());

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
                        CommandPaletteRenderRow::GroupPad => {
                            "group_pad".hash(&mut hasher);
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
            let pad_y = item_pad_y;
            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);

            let bg_hover = item_bg_hover(&theme);
            let bg_selected = alpha_mul(bg_hover, 0.85);
            let fg = theme.color_required("foreground");
            let fg_disabled = alpha_mul(fg, 0.5);
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
                        let fg = theme.color_required("muted-foreground");
                        let style = heading_text_style(&theme);
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout
                                },
                                padding: Edges {
                                    // new-york-v4: `px-2 py-1.5`.
                                    top: Px(6.0),
                                    right: pad_x,
                                    bottom: Px(6.0),
                                    left: pad_x,
                                },
                                ..Default::default()
                            },
                            move |cx| {
                                let mut text = ui::text(cx, heading)
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .nowrap()
                                    .text_color(ColorRef::Color(fg));

                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height);
                                }

                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }

                                vec![text.into_element(cx)]
                            },
                        )
                    }
                    CommandPaletteRenderRow::GroupPad => cx.container(
                        ContainerProps {
                            layout: {
                                let height = if group_next_top_pad_zero {
                                    group_pad_y
                                } else {
                                    Px(group_pad_y.0 * 2.0)
                                };
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Px(height);
                                layout
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    ),
                    CommandPaletteRenderRow::Separator => {
                        let border = border(&theme);
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(Px(1.0));
                                    // new-york-v4: `-mx-1 h-px`.
                                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                                    layout.margin.right =
                                        fret_ui::element::MarginEdge::Px(Px(-4.0));
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

                        let query_for_row = query_for_render.clone();
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
                                    cx.pressable_dispatch_command_if_enabled_opt(command);
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
                                        ..Default::default()
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
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                            },
                                            move |cx| {
                                                if !children.is_empty() {
                                                    return children;
                                                }

                                                let fg = if enabled { fg } else { fg_disabled };

                                                let left = cx.row(
                                                    RowProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.width = Length::Fill;
                                                            layout.size.min_width = Some(Px(0.0));
                                                            layout.flex.grow = 1.0;
                                                            layout.flex.shrink = 1.0;
                                                            layout.flex.basis = Length::Px(Px(0.0));
                                                            layout
                                                        },
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

                                                        out.push(cmdk_highlighted_label(
                                                            cx,
                                                            label.clone(),
                                                            query_for_row.as_ref(),
                                                            fg,
                                                            text_style.clone(),
                                                        ));

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
            let wrapper_h = theme
                .metric_by_key("component.command.input.wrapper_height")
                .unwrap_or(input_wrapper_h_fallback);
            let input_h = theme
                .metric_by_key("component.command.input.height")
                .unwrap_or(input_h_fallback);
            let icon_size = theme
                .metric_by_key("component.command.input.icon_size")
                .unwrap_or(input_icon_size_fallback);
            let pad_x = theme
                .metric_by_key("component.command.input.padding_x")
                .unwrap_or(Px(12.0));
            let gap = theme
                .metric_by_key("component.command.input.gap")
                .unwrap_or(Px(8.0));
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

            if matches!(wrapper.layout.size.height, Length::Auto) {
                wrapper.layout.size.height = Length::Px(wrapper_h);
            }
            wrapper.padding = Edges {
                top: Px(0.0),
                right: pad_x,
                bottom: Px(0.0),
                left: pad_x,
            };

            let a11y_label = self.a11y_label.clone();
            let input = command_text_input(
                cx,
                self.model.clone(),
                a11y_label,
                self.placeholder.clone(),
                self.input_role,
                active_descendant,
                self.input_expanded,
                input_h,
            );
            let input_id = input.id;

            let icon_fg = theme.color_required("muted-foreground");
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::SEARCH,
                Some(icon_size),
                Some(ColorRef::Color(icon_fg)),
            );
            let icon = cx.opacity(0.5, move |_cx| vec![icon]);

            let mut input = cx.row(
                RowProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                },
                move |_cx| vec![icon, input],
            );
            let list_labelled_by = Some(input_id.0);

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

            cx.key_on_key_down_for(input_id, key_handler);

            if disabled {
                input = cx.opacity(0.5, move |_cx| vec![input]);
                input = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        role: SemanticsRole::Generic,
                        disabled: true,
                        ..Default::default()
                    },
                    move |_cx| vec![input],
                );
            }

            let list = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::ListBox,
                    labelled_by_element: list_labelled_by,
                    ..Default::default()
                },
                move |cx| {
                    if row_ids.is_empty() {
                        let empty = self.empty_text;
                        return vec![CommandEmpty::new(empty).into_element(cx)];
                    }

                    let scroll = self.scroll;
                    let scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());
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
                            padding: Edges {
                                top: group_pad_y,
                                right: group_pad_x,
                                bottom: group_pad_y,
                                left: group_pad_x,
                            },
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
            );

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
    close_on_select: bool,
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
            .field("close_on_select", &self.close_on_select)
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
            close_on_select: true,
            empty_text: Arc::from("No results."),
        }
    }

    /// Creates a command dialog with controlled/uncontrolled models:
    /// - `open` / `default_open` (Dialog visibility)
    /// - `query` / `default_query` (cmdk input text)
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
        query: Option<Model<String>>,
        default_query: String,
        items: Vec<CommandItem>,
    ) -> Self {
        let open = radix_dialog::DialogRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        let query = controllable_state::use_controllable_model(cx, query, || default_query).model();
        Self::new(open, query, items)
    }

    pub fn new_with_host_commands<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
        query: Model<String>,
    ) -> Self {
        Self {
            open,
            query,
            entries: command_entries_from_host_commands(cx),
            a11y_label: None,
            disabled: false,
            wrap: true,
            close_on_select: true,
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

    /// Controls whether the dialog closes (and clears the query) after selecting an item.
    pub fn close_on_select(mut self, close_on_select: bool) -> Self {
        self.close_on_select = close_on_select;
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
        let open_model = open.clone();
        let query = self.query;
        let query_model = query.clone();
        let entries = self.entries;
        let a11y_label = self.a11y_label;
        let disabled = self.disabled;
        let wrap = self.wrap;
        let close_on_select = self.close_on_select;
        let empty_text = self.empty_text;

        Dialog::new(open).into_element(cx, trigger, move |cx| {
            // shadcn/ui v4: command dialog list is `max-h-[300px]` and is allowed to overflow the
            // viewport (the web implementation does not clamp it to the viewport height).
            let list_h = Px(300.0);

            let entries = if close_on_select {
                let close_action: fret_ui::action::OnActivate = Arc::new({
                    let open_model = open_model.clone();
                    let query_model = query_model.clone();
                    move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&open_model, |v| *v = false);
                        let _ = host.models_mut().update(&query_model, |v| v.clear());
                        host.request_redraw(action_cx.window);
                    }
                });

                entries
                    .into_iter()
                    .map(|entry| match entry {
                        CommandEntry::Item(mut item) => {
                            item.on_select = Some(match item.on_select.take() {
                                Some(prev) => {
                                    let close_action = close_action.clone();
                                    Arc::new(move |host, cx, reason| {
                                        prev(host, cx, reason);
                                        close_action(host, cx, reason);
                                    })
                                }
                                None => close_action.clone(),
                            });
                            CommandEntry::Item(item)
                        }
                        CommandEntry::Group(mut group) => {
                            group.items = group
                                .items
                                .into_iter()
                                .map(|mut item| {
                                    item.on_select = Some(match item.on_select.take() {
                                        Some(prev) => {
                                            let close_action = close_action.clone();
                                            Arc::new(move |host, cx, reason| {
                                                prev(host, cx, reason);
                                                close_action(host, cx, reason);
                                            })
                                        }
                                        None => close_action.clone(),
                                    });
                                    item
                                })
                                .collect();
                            CommandEntry::Group(group)
                        }
                        CommandEntry::Separator(sep) => CommandEntry::Separator(sep),
                    })
                    .collect()
            } else {
                entries
            };

            let palette = CommandPalette::new(query, Vec::new())
                .command_dialog_defaults()
                .entries(entries)
                .a11y_label(a11y_label.unwrap_or_else(|| Arc::from("Command palette")))
                .disabled(disabled)
                .wrap(wrap)
                .empty_text(empty_text)
                .refine_scroll_layout(
                    LayoutRefinement::default()
                        .h_px(MetricRef::Px(list_h))
                        .max_h(MetricRef::Px(list_h)),
                )
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

    use std::cell::RefCell;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, SemanticsRole, Size, SvgId,
        SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::{
        CommandScope, WindowCommandActionAvailabilityService, WindowCommandEnabledService,
    };
    use fret_ui::tree::UiTree;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        )
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
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

    #[test]
    fn command_palette_new_controllable_prefers_controlled_query_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let query = app.models_mut().insert(String::from("hello"));

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let palette = CommandPalette::new_controllable(
                cx,
                Some(query.clone()),
                String::from("ignored"),
                Vec::new(),
            );
            assert_eq!(palette.model, query);
        });
    }

    #[test]
    fn command_palette_new_controllable_applies_default_query() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let model_out: RefCell<Option<Model<String>>> = RefCell::new(None);

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let palette =
                CommandPalette::new_controllable(cx, None, String::from("hello"), Vec::new());
            *model_out.borrow_mut() = Some(palette.model.clone());
        });

        let model = model_out.borrow().clone().expect("query model");
        let value = model.read_ref(&app, |s| s.clone()).expect("read query");
        assert_eq!(value, "hello");
    }

    #[test]
    fn command_dialog_new_controllable_prefers_controlled_models() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::from("hello"));

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let dialog = CommandDialog::new_controllable(
                cx,
                Some(open.clone()),
                false,
                Some(query.clone()),
                String::from("ignored"),
                Vec::new(),
            );
            assert_eq!(dialog.open, open);
            assert_eq!(dialog.query, query);
        });
    }

    #[test]
    fn host_command_entries_respect_window_command_enabled_overrides() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.disabled-command");
        app.commands_mut()
            .register(cmd.clone(), CommandMeta::new("Disabled Command"));
        app.set_global(WindowCommandEnabledService::default());
        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        let disabled: RefCell<Option<bool>> = RefCell::new(None);
        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let entries = command_entries_from_host_commands(cx);
            for entry in entries {
                if let CommandEntry::Item(item) = entry
                    && item.command.as_ref() == Some(&cmd)
                {
                    *disabled.borrow_mut() = Some(item.disabled);
                    break;
                }
            }
        });

        assert_eq!(
            *disabled.borrow(),
            Some(true),
            "expected the command entry to be disabled via WindowCommandEnabledService"
        );
    }

    #[test]
    fn host_command_entries_respect_widget_action_availability_snapshot() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), false);
                svc.set_snapshot(window, snapshot);
            },
        );

        let disabled: RefCell<Option<bool>> = RefCell::new(None);
        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let entries = command_entries_from_host_commands(cx);
            for entry in entries {
                if let CommandEntry::Item(item) = entry
                    && item.command.as_ref() == Some(&cmd)
                {
                    *disabled.borrow_mut() = Some(item.disabled);
                    break;
                }
            }
        });

        assert_eq!(
            *disabled.borrow(),
            Some(true),
            "expected the command entry to be disabled via WindowCommandActionAvailabilityService"
        );
    }

    #[test]
    fn host_command_entries_prefer_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        app.set_global(fret_runtime::WindowCommandGatingService::default());
        app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, app| {
                let input_ctx = command_palette_input_context(app);
                let enabled_overrides: HashMap<CommandId, bool> = HashMap::new();
                let mut availability: HashMap<CommandId, bool> = HashMap::new();
                availability.insert(cmd.clone(), false);
                svc.set_snapshot(
                    window,
                    WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
                        .with_action_availability(Some(Arc::new(availability))),
                );
            },
        );

        let disabled: RefCell<Option<bool>> = RefCell::new(None);
        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
            let entries = command_entries_from_host_commands(cx);
            for entry in entries {
                if let CommandEntry::Item(item) = entry
                    && item.command.as_ref() == Some(&cmd)
                {
                    *disabled.borrow_mut() = Some(item.disabled);
                    break;
                }
            }
        });

        assert_eq!(
            *disabled.borrow(),
            Some(true),
            "expected the command entry to be disabled via WindowCommandGatingService snapshot"
        );
    }

    fn click(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        at: Point,
    ) {
        ui.dispatch_event(
            app,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: at,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            app,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: at,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
    }

    fn rect_center(r: Rect) -> Point {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 * 0.5),
            Px(r.origin.y.0 + r.size.height.0 * 0.5),
        )
    }

    fn render_dialog_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        query: Model<String>,
        items: Vec<CommandItem>,
        close_on_select: bool,
    ) -> fret_core::NodeId {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        crate::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "cmdk-dialog",
            |cx| {
                vec![
                    CommandDialog::new(open, query, items)
                        .close_on_select(close_on_select)
                        .into_element(cx, |cx| crate::Button::new("Open").into_element(cx)),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn command_dialog_close_on_select_closes_and_clears_query() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::from("al"));
        let selected = app.models_mut().insert(false);

        let on_select: fret_ui::action::OnActivate = Arc::new({
            let selected = selected.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&selected, |v| *v = true);
                host.request_redraw(action_cx.window);
            }
        });

        let items = vec![CommandItem::new("Alpha").on_select_action(on_select)];

        let bounds = bounds();
        let mut services = FakeServices::default();

        let _root = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items.clone(),
            true,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Alpha"))
            .map(|n| n.bounds)
            .expect("Alpha option bounds");

        click(&mut ui, &mut app, &mut services, rect_center(alpha));

        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items,
            true,
        );

        assert!(!open.read_ref(&app, |v| *v).expect("read open"));
        assert_eq!(query.read_ref(&app, |v| v.clone()).expect("read query"), "");
        assert!(selected.read_ref(&app, |v| *v).expect("read selected"));
    }

    #[test]
    fn command_dialog_close_on_select_false_does_not_close_or_clear_query() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::from("al"));
        let selected = app.models_mut().insert(false);

        let on_select: fret_ui::action::OnActivate = Arc::new({
            let selected = selected.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&selected, |v| *v = true);
                host.request_redraw(action_cx.window);
            }
        });

        let items = vec![CommandItem::new("Alpha").on_select_action(on_select)];

        let bounds = bounds();
        let mut services = FakeServices::default();

        let _root = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items.clone(),
            false,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Alpha"))
            .map(|n| n.bounds)
            .expect("Alpha option bounds");

        click(&mut ui, &mut app, &mut services, rect_center(alpha));

        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items,
            false,
        );

        assert!(open.read_ref(&app, |v| *v).expect("read open"));
        assert_eq!(
            query.read_ref(&app, |v| v.clone()).expect("read query"),
            "al"
        );
        assert!(selected.read_ref(&app, |v| *v).expect("read selected"));
    }

    #[test]
    fn command_dialog_list_can_overflow_viewport_in_tight_heights() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(140.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<CommandItem> = (0..80)
            .map(|i| CommandItem::new(format!("Item {i}")))
            .collect();

        // First frame: mount overlay/content.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items.clone(),
            true,
        );
        // Second/third frame: settle overlay layout and list metrics.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            items.clone(),
            true,
        );
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            query,
            items,
            true,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        let list_bounds = ui.debug_node_bounds(list.id).expect("listbox bounds");
        let list_bottom = list_bounds.origin.y.0 + list_bounds.size.height.0;

        assert!(
            list_bounds.size.height.0 >= 299.0,
            "expected fixed 300px listbox height; list={list_bounds:?}"
        );
        assert!(
            list_bounds.origin.y.0 < 0.0 || list_bottom > bounds.size.height.0 + 0.01,
            "expected listbox to overflow tight viewports; list={list_bounds:?} viewport={bounds:?}"
        );
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
                CommandPaletteRenderRow::GroupPad => "P".to_string(),
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
            .find(|n| n.role == SemanticsRole::ComboBox && n.id == focus)
            .expect("focused combobox node");

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        assert!(
            list.labelled_by.iter().any(|id| *id == input.id),
            "listbox should be labelled by the focused input"
        );
        assert!(
            input.controls.iter().any(|id| *id == list.id),
            "focused input should control the listbox"
        );

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
            .find(|n| n.role == SemanticsRole::ComboBox && n.id == focus)
            .expect("focused combobox node");

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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(
                    Px(beta_bounds.origin.x.0 + 1.0),
                    Px(beta_bounds.origin.y.0 + 1.0),
                ),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
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
            .find(|n| n.role == SemanticsRole::ComboBox && n.id == focus)
            .expect("focused combobox node");

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
            .find(|n| n.role == SemanticsRole::ComboBox && n.id == focus)
            .expect("focused combobox node");
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
    fn command_palette_sort_orders_ungrouped_items_by_score() {
        let entries = vec![
            CommandItem::new("pal").into(),
            CommandItem::new("alpha").into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "al");
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["I:alpha".to_string(), "I:pal".to_string()]
        );
    }

    #[test]
    fn command_palette_sort_orders_groups_by_best_item_score() {
        let entries = vec![
            CommandGroup::new(vec![CommandItem::new("pal")])
                .heading("g1")
                .into(),
            CommandGroup::new(vec![CommandItem::new("alpha")])
                .heading("g2")
                .into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "al");
        assert_eq!(
            row_signatures(&rows, &items),
            vec![
                "H:g2".to_string(),
                "I:alpha".to_string(),
                "P".to_string(),
                "H:g1".to_string(),
                "I:pal".to_string()
            ]
        );
    }

    #[test]
    fn command_palette_sort_pushes_groups_below_ungrouped_items() {
        let entries = vec![
            CommandGroup::new(vec![CommandItem::new("pal")])
                .heading("g1")
                .into(),
            CommandItem::new("alpha").into(),
        ];

        let (rows, items) = command_palette_render_rows_for_query(entries, "al");
        assert_eq!(
            row_signatures(&rows, &items),
            vec![
                "I:alpha".to_string(),
                "H:g1".to_string(),
                "I:pal".to_string()
            ]
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
