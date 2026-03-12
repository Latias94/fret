use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, KeyCode, NodeId, Px, SemanticsRole, TextStyle,
};
use fret_icons::{IconId, ids};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_runtime::WindowCommandGatingSnapshot;
use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, RowProps,
    SemanticsDecoration, SizeStyle, TextInputProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, TextInputStyle, Theme, ThemeSnapshot, UiHost};
use fret_ui_headless::cmdk_score;
use fret_ui_headless::cmdk_selection;
use fret_ui_kit::command::{
    CommandCatalogEntry as UiKitCommandCatalogEntry,
    CommandCatalogGroup as UiKitCommandCatalogGroup, CommandCatalogItem as UiKitCommandCatalogItem,
};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::active_descendant as active_desc;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::dialog as radix_dialog;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::layout as shadcn_layout;
use crate::rtl;
use crate::shortcut_display::shortcut_text_element;
use crate::{Dialog, DialogContent, ScrollArea};

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;
type OnOpenChangeWithReason =
    Arc<dyn Fn(bool, CommandDialogOpenChangeReason) + Send + Sync + 'static>;
type OnValueChange = Arc<dyn Fn(Option<Arc<str>>) + Send + Sync + 'static>;
type OnSelectValueAction = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiActionHost,
            fret_ui::action::ActionCx,
            ActivateReason,
            Arc<str>,
        ) + 'static,
>;

type CommandPaletteFilter = dyn Fn(&str, &str, &[&str]) -> f32 + Send + Sync + 'static;
pub type CommandPaletteFilterFn = Arc<CommandPaletteFilter>;

/// Open-change reasons aligned with Base UI dialog semantics for command dialog usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDialogOpenChangeReason {
    TriggerPress,
    OutsidePress,
    ItemPress,
    EscapeKey,
    FocusOut,
    None,
}

fn command_dialog_open_change_reason_from_dismiss_reason(
    reason: fret_ui::action::DismissReason,
) -> CommandDialogOpenChangeReason {
    match reason {
        fret_ui::action::DismissReason::Escape => CommandDialogOpenChangeReason::EscapeKey,
        fret_ui::action::DismissReason::OutsidePress { .. } => {
            CommandDialogOpenChangeReason::OutsidePress
        }
        fret_ui::action::DismissReason::FocusOutside => CommandDialogOpenChangeReason::FocusOut,
        fret_ui::action::DismissReason::Scroll => CommandDialogOpenChangeReason::None,
    }
}

#[derive(Debug, Clone)]
struct PendingCommandDispatch {
    command: CommandId,
    reason: ActivateReason,
}

pub use fret_ui_kit::command::CommandCatalogOptions;

pub fn command_entries_from_host_commands<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Vec<CommandEntry> {
    command_entries_from_host_commands_with_options(cx, CommandCatalogOptions::default())
}

pub fn command_entries_from_host_commands_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
) -> Vec<CommandEntry> {
    fret_ui_kit::command::command_catalog_entries_from_host_commands_with_options(cx, options)
        .into_iter()
        .map(Into::into)
        .collect()
}

pub fn command_entries_from_host_commands_with_gating_snapshot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
    gating: &WindowCommandGatingSnapshot,
) -> Vec<CommandEntry> {
    fret_ui_kit::command::command_catalog_entries_from_host_commands_with_gating_snapshot(
        cx, options, gating,
    )
    .into_iter()
    .map(Into::into)
    .collect()
}

fn border(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .expect("missing theme token: border/input")
}

fn bg(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("background"))
        .expect("missing theme token: popover/background")
}

fn item_bg_hover(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("muted"))
        .expect("missing theme token: accent/muted")
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn cmdk_trimmed_arc(value: Arc<str>) -> Arc<str> {
    let trimmed = value.trim();
    if trimmed == value.as_ref() {
        value
    } else {
        Arc::<str>::from(trimmed)
    }
}

fn sanitize_test_id_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;

    for ch in raw.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        out.push_str("item");
    }

    out
}

fn command_text_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Arc<str>,
    placeholder: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    test_id: Option<Arc<str>>,
    active_descendant: Option<NodeId>,
    expanded: Option<bool>,
    pad_y: Px,
    height: Length,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let fg = theme.color_token("popover-foreground");
    let placeholder_fg = theme.color_token("muted-foreground");

    let mut chrome = TextInputStyle::from_theme(theme.clone());
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
    // Prefer browser-like IME composition visuals: keep the composed text in the normal color and
    // rely on underline for feedback (instead of tinting preedit with the theme "primary" color).
    chrome.preedit_color = chrome.text_color;
    chrome.preedit_underline_color = chrome.text_color;

    let mut props = TextInputProps::new(model);
    props.a11y_label = Some(a11y_label);
    props.a11y_role = a11y_role;
    props.test_id = test_id;
    props.placeholder = placeholder;
    props.active_descendant = active_descendant;
    props.expanded = expanded;
    props.chrome = chrome;
    props.text_style = item_text_style(&theme);
    props.layout.size = SizeStyle {
        width: Length::Auto,
        height,
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    props.layout.flex.grow = 1.0;
    props.layout.flex.shrink = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));
    props.layout.overflow = Overflow::Clip;

    cx.text_input(props)
}

fn cmdk_highlighted_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    query: &str,
    fg: Color,
    nonmatch_fg: Color,
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
        return apply_text_style(ui::text(label))
            .layout(LayoutRefinement::default().min_w_0().flex_1())
            .text_color(ColorRef::Color(fg))
            .into_element(cx);
    }

    let ranges = cmdk_score::command_match_ranges(label.as_ref(), query);
    if ranges.is_empty() {
        return apply_text_style(ui::text(label))
            .layout(LayoutRefinement::default().min_w_0().flex_1())
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
                apply_text_style(ui::text(text))
                    .text_color(ColorRef::Color(nonmatch_fg))
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
                apply_text_style(ui::text(text))
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
            apply_text_style(ui::text(text))
                .text_color(ColorRef::Color(nonmatch_fg))
                .into_element(cx),
        );
    }

    cx.row(
        RowProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.min_width = Some(Length::Px(Px(0.0)));
                layout.flex.grow = 1.0;
                layout.flex.shrink = 1.0;
                layout.flex.basis = Length::Px(Px(0.0));
                layout
            },
            gap: Px(0.0).into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
        },
        move |_cx| pieces,
    )
}

pub(crate) fn item_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.command.item.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.command.item.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::NORMAL;
    style
}

fn heading_text_style(theme: &ThemeSnapshot) -> TextStyle {
    // shadcn/ui v4: command group headings use `text-xs` / `leading-4`.
    let size = theme
        .metric_by_key("component.command.heading.text_px")
        .unwrap_or(Px(12.0));
    let line_height = theme
        .metric_by_key("component.command.heading.line_height")
        .unwrap_or(Px(16.0));

    let mut style = typography::fixed_line_box_style(FontId::ui(), size, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

pub(crate) fn shortcut_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let base_size = theme.metric_token("font.size");
    let base_line_height = theme.metric_token("font.line_height");

    let px = theme
        .metric_by_key("component.command.shortcut.text_px")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
        .unwrap_or_else(|| Px((base_size.0 - 2.0).max(10.0)));
    let line_height = theme
        .metric_by_key("component.command.shortcut.line_height")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT))
        .unwrap_or_else(|| Px((base_line_height.0 - 4.0).max(px.0)));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::NORMAL;
    // new-york-v4: `tracking-widest`.
    style.letter_spacing_em = Some(0.10);
    style
}

/// shadcn/ui `CommandShortcut` (v4).
#[derive(Clone)]
pub struct CommandShortcut {
    text: Arc<str>,
    auto_margin_inline_start: bool,
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
        Self {
            text: text.into(),
            auto_margin_inline_start: true,
        }
    }

    /// Renders the shortcut without applying an inline-start `auto` margin.
    ///
    /// This is useful when the caller is already handling RTL mirroring and spacing explicitly.
    pub fn inline(mut self) -> Self {
        self.auto_margin_inline_start = false;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("muted-foreground");
        let style = shortcut_text_style(&theme);
        let dir = crate::use_direction(cx, None);
        let shortcut_layout = if self.auto_margin_inline_start {
            rtl::layout_refinement_margin_inline_start_auto(dir)
        } else {
            LayoutRefinement::default()
        }
        .flex_shrink_0();

        shortcut_text_element(cx, &theme, self.text, style, fg, shortcut_layout)
    }
}

#[cfg(test)]
mod command_shortcut_tests {
    use super::*;
    use crate::shortcut_display::shortcut_needs_symbol_font;

    #[test]
    fn shortcut_symbol_detection() {
        assert!(shortcut_needs_symbol_font("⌘P"));
        assert!(shortcut_needs_symbol_font("⌥⇧K"));
        assert!(!shortcut_needs_symbol_font("Ctrl+P"));
        assert!(!shortcut_needs_symbol_font("Alt+Shift+K"));
    }
}

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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
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
        let (props, fg_root) = {
            let theme = Theme::global(&*cx.app).snapshot();
            let base = ChromeRefinement::default()
                .rounded(Radius::Lg)
                .merge(
                    ChromeRefinement::default()
                        .border_width(Px(1.0))
                        .border_color(ColorRef::Color(border(&theme)))
                        .bg(ColorRef::Color(bg(&theme))),
                )
                .merge(self.chrome);
            (
                decl_style::container_props(
                    &theme,
                    base,
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .overflow_hidden()
                        .merge(self.layout),
                ),
                theme.color_token("popover-foreground"),
            )
        };
        let content = ui::v_flex(move |_cx| self.children)
            .gap(Space::N0)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);
        let children = vec![current_color::scope_element(
            cx,
            ColorRef::Color(fg_root),
            content,
        )];
        shadcn_layout::container_flow_fill_width(cx, props, children)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandInputHeightMode {
    /// Use the shadcn theme metric defaults (`component.command.input.*`).
    ThemePx,
    /// Let layout size to content (web-like `h-auto`).
    Auto,
}

#[derive(Clone)]
pub struct CommandInput {
    model: fret_runtime::Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    input_test_id: Option<Arc<str>>,
    disabled: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    wrapper_height_mode: CommandInputHeightMode,
    input_height_mode: CommandInputHeightMode,
    input_padding_y: Option<MetricRef>,
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
            input_test_id: None,
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            wrapper_height_mode: CommandInputHeightMode::ThemePx,
            input_height_mode: CommandInputHeightMode::ThemePx,
            input_padding_y: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Installs a stable `test_id` on the underlying text input (not the wrapper row).
    pub fn input_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.input_test_id = Some(test_id.into());
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

    /// Web-like `h-auto` for the wrapper row (`data-slot=command-input-wrapper`).
    pub fn wrapper_height_auto(mut self) -> Self {
        self.wrapper_height_mode = CommandInputHeightMode::Auto;
        self
    }

    /// Web-like `h-auto` for the underlying text input (`data-slot=command-input`).
    pub fn input_height_auto(mut self) -> Self {
        self.input_height_mode = CommandInputHeightMode::Auto;
        self
    }

    /// Overrides the text input's vertical padding (Tailwind-like `py-*`).
    pub fn input_padding_y(mut self, pad_y: impl Into<MetricRef>) -> Self {
        self.input_padding_y = Some(pad_y.into());
        self
    }

    /// Convenience for `input_padding_y(Px(...))`.
    pub fn input_padding_y_px(self, pad_y: Px) -> Self {
        self.input_padding_y(pad_y)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.watch_model(&self.model).observe();
            let theme = Theme::global(&*cx.app).snapshot();
            let theme = &theme;

            let border = border(theme);
            let disabled = self.disabled;
            let wrapper_h = theme
                .metric_by_key("component.command.input.wrapper_height")
                .unwrap_or(Px(36.0));
            let input_h = theme
                .metric_by_key("component.command.input.height")
                .unwrap_or(Px(40.0));
            let input_pad_y = self
                .input_padding_y
                .unwrap_or_else(|| MetricRef::space(Space::N3))
                .resolve(theme);
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
                .map(|m| m.resolve(theme))
                .unwrap_or(Px(1.0));
            let border_color = chrome
                .border_color
                .as_ref()
                .map(|c| c.resolve(theme))
                .unwrap_or(border);
            let background = chrome.background.as_ref().map(|c| c.resolve(theme));
            let radius = chrome
                .radius
                .as_ref()
                .map(|m| m.resolve(theme))
                .unwrap_or(Px(0.0));

            let padding = chrome.padding.clone().unwrap_or_default();
            let pad_top = padding.top.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
            let pad_right = padding.right.map(|m| m.resolve(theme)).unwrap_or(pad_x);
            let pad_bottom = padding.bottom.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
            let pad_left = padding.left.map(|m| m.resolve(theme)).unwrap_or(pad_x);
            let dir = crate::use_direction(cx, None);

            let icon_fg = theme.color_token("popover-foreground");

            let mut wrapper = decl_style::container_props(
                theme,
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
            if self.wrapper_height_mode == CommandInputHeightMode::ThemePx
                && matches!(wrapper.layout.size.height, Length::Auto)
            {
                wrapper.layout.size.height = Length::Px(wrapper_h);
            }
            wrapper.padding = rtl::padding_edges_with_inline_start_end(
                dir, pad_top, pad_bottom, pad_left, pad_right,
            )
            .into();

            cx.container(wrapper, move |cx| {
                let a11y_label = self
                    .a11y_label
                    .clone()
                    .unwrap_or_else(|| Arc::from("Command input"));
                let placeholder = self.placeholder.clone();
                let input_test_id = self.input_test_id.clone();

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
                    input_test_id,
                    None,
                    None,
                    input_pad_y,
                    match self.input_height_mode {
                        CommandInputHeightMode::ThemePx => Length::Px(input_h),
                        CommandInputHeightMode::Auto => Length::Auto,
                    },
                );

                let row_height = if self.wrapper_height_mode == CommandInputHeightMode::Auto {
                    Length::Auto
                } else {
                    Length::Fill
                };
                let mut row = cx.row(
                    RowProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = row_height;
                            layout
                        },
                        gap: gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                    },
                    move |_cx| {
                        let (a, b) = rtl::inline_start_end_pair(dir, icon, input);
                        vec![a, b]
                    },
                );

                if disabled {
                    row = cx.opacity(0.5, move |_cx| vec![row]);
                }

                let mut input = row;
                if disabled {
                    input = input.attach_semantics(SemanticsDecoration {
                        role: Some(SemanticsRole::Generic),
                        disabled: Some(true),
                        ..Default::default()
                    });
                }
                vec![input]
            })
        })
    }
}

pub struct CommandItem {
    label: Arc<str>,
    value: Arc<str>,
    disabled: bool,
    force_mount: bool,
    keywords: Vec<Arc<str>>,
    checked: bool,
    show_checkmark: bool,
    test_id: Option<Arc<str>>,
    shortcut: Option<Arc<str>>,
    command: Option<CommandId>,
    on_select: Option<fret_ui::action::OnActivate>,
    on_select_value: Option<OnSelectValueAction>,
    leading_icon: Option<IconId>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandItem")
            .field("label", &self.label.as_ref())
            .field("value", &self.value.as_ref())
            .field("disabled", &self.disabled)
            .field("force_mount", &self.force_mount)
            .field("keywords_len", &self.keywords.len())
            .field("checked", &self.checked)
            .field("show_checkmark", &self.show_checkmark)
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("shortcut", &self.shortcut.as_ref().map(|s| s.as_ref()))
            .field("command", &self.command)
            .field("on_select", &self.on_select.is_some())
            .field("on_select_value", &self.on_select_value.is_some())
            .field("leading_icon", &self.leading_icon)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CommandItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: cmdk_trimmed_arc(label.clone()),
            disabled: false,
            force_mount: false,
            keywords: Vec::new(),
            checked: false,
            show_checkmark: false,
            test_id: None,
            shortcut: None,
            command: None,
            on_select: None,
            on_select_value: None,
            leading_icon: None,
            children: Vec::new(),
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = cmdk_trimmed_arc(value.into());
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
        self.keywords = keywords
            .into_iter()
            .map(|kw| cmdk_trimmed_arc(kw.into()))
            .collect();
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

    /// cmdk: `forceMount`. When true, the item remains rendered even when it does not match the
    /// current filter query.
    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    /// Prefer this over `children([icon(cx, ...), ...])` so the icon can follow the row's
    /// foreground (`currentColor`) for hover/active/disabled states.
    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Shows a cmdk-style checkmark indicator, with visibility controlled by `checked`.
    pub fn checkmark(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.show_checkmark = true;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
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

    /// cmdk: `onSelect(value)`. Called when the item is activated, with the item's resolved `value`.
    pub fn on_select_value_action<F>(mut self, on_select: F) -> Self
    where
        F: Fn(
                &mut dyn fret_ui::action::UiActionHost,
                fret_ui::action::ActionCx,
                ActivateReason,
                Arc<str>,
            ) + 'static,
    {
        self.on_select_value = Some(Arc::new(on_select));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }
}

/// shadcn/ui `CommandGroup` (v4).
pub struct CommandGroup {
    heading: Option<Arc<str>>,
    items: Vec<CommandItem>,
    force_mount: bool,
}

impl std::fmt::Debug for CommandGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandGroup")
            .field("heading", &self.heading.as_ref().map(|s| s.as_ref()))
            .field("items_len", &self.items.len())
            .field("force_mount", &self.force_mount)
            .finish()
    }
}

impl CommandGroup {
    pub fn new(items: impl IntoIterator<Item = CommandItem>) -> Self {
        let items = items.into_iter().collect();
        Self {
            heading: None,
            items,
            force_mount: false,
        }
    }

    pub fn heading(mut self, heading: impl Into<Arc<str>>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// cmdk: `forceMount`. When true, keeps the group visible even when the query filters out all items.
    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
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
#[derive(Debug, Clone, Default)]
pub struct CommandSeparator {
    always_render: bool,
    test_id: Option<Arc<str>>,
}

impl CommandSeparator {
    pub fn new() -> Self {
        Self::default()
    }

    /// cmdk: `alwaysRender`. When true, the separator remains visible even when the query is non-empty.
    pub fn always_render(mut self, always_render: bool) -> Self {
        self.always_render = always_render;
        self
    }

    /// Installs a stable `test_id` on the separator (for automation).
    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, text_style) = {
            let theme = Theme::global(&*cx.app).snapshot();
            let fg = theme.color_token("muted-foreground");
            let text_style = item_text_style(&theme);
            (fg, text_style)
        };
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
                }
                .into(),
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
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                    },
                    move |cx| {
                        let mut text = ui::text(self.text.clone())
                            .text_size_px(text_style.size)
                            .font_weight(text_style.weight)
                            .nowrap()
                            .text_color(ColorRef::Color(fg));

                        if let Some(line_height) = text_style.line_height {
                            text = text.line_height_px(line_height).line_height_policy(
                                fret_core::TextLineHeightPolicy::FixedFromStyle,
                            );
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

/// cmdk `Command.Loading` (list row).
#[derive(Clone)]
pub struct CommandLoading {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
    progress: Option<u8>,
}

impl std::fmt::Debug for CommandLoading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandLoading")
            .field("text", &self.text.as_ref())
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("progress", &self.progress)
            .finish()
    }
}

impl CommandLoading {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
            progress: None,
        }
    }

    /// Installs a stable `test_id` on the loading row (for automation).
    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    /// Optional progress percent (0..=100). When absent, the loading row is indeterminate.
    pub fn progress(mut self, progress: u8) -> Self {
        self.progress = Some(progress.min(100));
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let text_for_semantics = self.text.clone();
        let text_for_render = self.text.clone();
        let test_id = self.test_id.clone();
        let progress = self.progress;

        let (fg, text_style) = {
            let theme = Theme::global(&*cx.app).snapshot();
            let fg = theme.color_token("muted-foreground");
            let text_style = item_text_style(&theme);
            (fg, text_style)
        };

        let mut row = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                // new-york-v4: keep parity with `CommandEmpty` spacing.
                padding: Edges {
                    top: Px(24.0),
                    right: Px(0.0),
                    bottom: Px(24.0),
                    left: Px(0.0),
                }
                .into(),
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
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                    },
                    move |cx| {
                        let mut text = ui::text(text_for_render.clone())
                            .text_size_px(text_style.size)
                            .font_weight(text_style.weight)
                            .nowrap()
                            .text_color(ColorRef::Color(fg));

                        if let Some(line_height) = text_style.line_height {
                            text = text.line_height_px(line_height).line_height_policy(
                                fret_core::TextLineHeightPolicy::FixedFromStyle,
                            );
                        }

                        if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                            text = text.letter_spacing_em(letter_spacing_em);
                        }

                        vec![text.into_element(cx)]
                    },
                )]
            },
        );

        let mut a11y = SemanticsDecoration::default()
            .role(SemanticsRole::ProgressBar)
            .label(text_for_semantics)
            .numeric_range(0.0, 100.0);
        if let Some(progress) = progress {
            a11y = a11y
                .value(Arc::<str>::from(format!("{progress}%")))
                .numeric_value(progress as f64);
        }
        row = row.attach_semantics(a11y);
        if let Some(test_id) = test_id {
            row = row.test_id(test_id);
        }
        row
    }
}

pub enum CommandEntry {
    Item(CommandItem),
    Group(CommandGroup),
    Separator(CommandSeparator),
    Loading(CommandLoading),
}

impl From<CommandItem> for CommandEntry {
    fn from(value: CommandItem) -> Self {
        Self::Item(value)
    }
}

impl From<CommandLoading> for CommandEntry {
    fn from(value: CommandLoading) -> Self {
        Self::Loading(value)
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

impl From<UiKitCommandCatalogItem> for CommandItem {
    fn from(value: UiKitCommandCatalogItem) -> Self {
        let mut item = CommandItem::new(value.label)
            .value(value.value)
            .keywords(value.keywords)
            .disabled(value.disabled)
            .on_select(value.command);
        if let Some(shortcut) = value.shortcut {
            item = item.shortcut(shortcut);
        }
        item
    }
}

impl From<UiKitCommandCatalogGroup> for CommandGroup {
    fn from(value: UiKitCommandCatalogGroup) -> Self {
        CommandGroup::new(value.items.into_iter().map(Into::into)).heading(value.heading)
    }
}

impl From<UiKitCommandCatalogEntry> for CommandEntry {
    fn from(value: UiKitCommandCatalogEntry) -> Self {
        match value {
            UiKitCommandCatalogEntry::Item(item) => CommandEntry::Item(item.into()),
            UiKitCommandCatalogEntry::Group(group) => CommandEntry::Group(group.into()),
        }
    }
}

pub struct CommandList {
    entries: Vec<CommandEntry>,
    disabled: bool,
    empty_text: Arc<str>,
    query: Option<Model<String>>,
    highlight_query: Option<Model<String>>,
    scroll: LayoutRefinement,
}

impl std::fmt::Debug for CommandList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandList")
            .field("entries_len", &self.entries.len())
            .field("disabled", &self.disabled)
            .field("empty_text", &self.empty_text.as_ref())
            .field("query", &self.query.is_some())
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl CommandList {
    pub fn new(items: impl IntoIterator<Item = CommandItem>) -> Self {
        let entries = items.into_iter().map(CommandEntry::Item).collect();
        Self {
            entries,
            disabled: false,
            empty_text: Arc::from("No results."),
            query: None,
            highlight_query: None,
            scroll: LayoutRefinement::default()
                .max_h(Px(300.0))
                .w_full()
                .min_w_0(),
        }
    }

    pub fn new_entries(entries: impl IntoIterator<Item = CommandEntry>) -> Self {
        let entries = entries.into_iter().collect();
        Self {
            entries,
            ..Self::new([])
        }
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = CommandEntry>) -> Self {
        self.entries = entries.into_iter().collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    /// Provide a query model for cmdk-style fuzzy filtering and sorting.
    ///
    /// This is a smaller surface than `CommandPalette`: it filters the static entry list, but it
    /// does not adopt `active_descendant` input semantics.
    pub fn query_model(mut self, model: Model<String>) -> Self {
        self.query = Some(model);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let disabled = self.disabled;
            let entries = self.entries;
            let query_model = self.query;
            let highlight_query = self.highlight_query;

            // Note: `CommandList` is a simple list rendering helper (legacy roving-style semantics).
            // `CommandPalette` is the cmdk-style implementation that keeps focus in the input and
            // drives highlight via `active_descendant` (ADR 0073).
            let query_for_filter: Arc<str> = query_model
                .as_ref()
                .and_then(|model| {
                    cx.watch_model(model)
                        .layout()
                        .read_ref(|s| s.trim().to_ascii_lowercase())
                        .ok()
                })
                .map(|trimmed| Arc::<str>::from(trimmed))
                .unwrap_or_else(|| Arc::from(""));

            let should_filter = query_model.is_some();
            let (render_rows, items, _item_groups) =
                command_palette_render_rows_for_query_with_options(
                    entries,
                    query_for_filter.as_ref(),
                    should_filter,
                    None,
                );
            let list_busy = render_rows
                .iter()
                .any(|row| matches!(row, CommandPaletteRenderRow::Loading(_)));

            let query_for_render: Arc<str> = highlight_query
                .as_ref()
                .or(query_model.as_ref())
                .and_then(|model| {
                    cx.watch_model(model)
                        .layout()
                        .read_ref(|s| s.trim().to_ascii_lowercase())
                        .ok()
                })
                .and_then(|trimmed| (!trimmed.is_empty()).then(|| Arc::<str>::from(trimmed)))
                .unwrap_or_else(|| Arc::from(""));

            if items.is_empty() {
                let empty = self.empty_text;
                return CommandEmpty::new(empty).into_element(cx);
            }

            let disabled_flags: Vec<bool> = items
                .iter()
                .map(|item| {
                    let Some(item) = item.as_ref() else {
                        return true;
                    };
                    disabled
                        || item.disabled
                        || (item.command.is_none()
                            && item.on_select.is_none()
                            && item.on_select_value.is_none())
                })
                .collect();
            let tab_stop = roving_focus_group::first_enabled(&disabled_flags);

            let mut items = items;

            let roving = RovingFocusProps {
                enabled: !disabled,
                wrap: true,
                disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
                ..Default::default()
            };

            let (
                row_gap,
                pad_x,
                pad_y,
                radius,
                ring,
                bg_hover,
                fg_selected,
                fg,
                fg_disabled,
                icon_fg,
                icon_fg_disabled,
                text_style,
                item_layout,
            ) = {
                let theme = Theme::global(&*cx.app).snapshot();
                let row_h = MetricRef::space(Space::N8).resolve(&theme);
                let row_gap = MetricRef::space(Space::N2).resolve(&theme);
                let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                // new-york-v4: `py-1.5` for `CommandItem` in the base `Command` surface.
                let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                let radius = MetricRef::radius(Radius::Sm).resolve(&theme);
                let ring = decl_style::focus_ring(&theme, radius);
                let bg_hover = item_bg_hover(&theme);
                let fg_selected = theme.color_token("accent-foreground");
                let fg = theme.color_token("foreground");
                let fg_disabled = alpha_mul(fg, 0.5);
                let icon_fg = theme.color_token("muted-foreground");
                let icon_fg_disabled = alpha_mul(icon_fg, 0.5);
                let text_style = item_text_style(&theme);
                let item_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().min_h(row_h).min_w_0(),
                );
                (
                    row_gap,
                    pad_x,
                    pad_y,
                    radius,
                    ring,
                    bg_hover,
                    fg_selected,
                    fg,
                    fg_disabled,
                    icon_fg,
                    icon_fg_disabled,
                    text_style,
                    item_layout,
                )
            };

            let scroll = self.scroll.w_full().min_w_0();

            let theme = Theme::global(&*cx.app).snapshot();
            let border = border(&theme);
            let heading_style = heading_text_style(&theme);
            let fg_heading = theme.color_token("muted-foreground");
            let group_pad_y = MetricRef::space(Space::N1).resolve(&theme);

            ScrollArea::new(vec![cx.roving_flex(
                RovingFlexProps {
                    flex: FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.min_height = Some(Length::Px(Px(0.0)));
                            layout
                        },
                        direction: fret_core::Axis::Vertical,
                        gap: Px(0.0).into(),
                        // new-york-v4: `CommandList` uses `scroll-py-1` and is typically
                        // wrapped in `CommandGroup` which uses `p-1`.
                        padding: Edges::all(Px(4.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                        ..Default::default()
                    },
                    roving,
                },
                move |cx| {
                    cx.roving_nav_apg();
                    let mut out = Vec::with_capacity(render_rows.len());

                    for row in render_rows.into_iter() {
                        match row {
                            CommandPaletteRenderRow::Heading(heading) => {
                                let heading = heading.clone();
                                let heading_style = heading_style.clone();
                                let fg_heading = fg_heading;
                                out.push(
                                    cx.container(
                                        ContainerProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Fill;
                                                layout
                                            },
                                            padding: Edges {
                                                top: Px(6.0),
                                                right: pad_x,
                                                bottom: Px(6.0),
                                                left: pad_x,
                                            }
                                            .into(),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let mut text = ui::text( heading)
                                                .text_size_px(heading_style.size)
                                                .font_weight(heading_style.weight)
                                                .nowrap()
                                                .text_color(ColorRef::Color(fg_heading));

                                            if let Some(line_height) = heading_style.line_height {
                                                text = text
                                                    .line_height_px(line_height)
                                                    .line_height_policy(
                                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                );
                                            }
                                            if let Some(letter_spacing_em) =
                                                heading_style.letter_spacing_em
                                            {
                                                text = text.letter_spacing_em(letter_spacing_em);
                                            }

                                            vec![text.into_element(cx)]
                                        },
                                    ),
                                );
                            }
                            CommandPaletteRenderRow::GroupPad => out.push(cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Px(group_pad_y);
                                        layout
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            )),
                            CommandPaletteRenderRow::Separator(test_id) => {
                                let mut sep = cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Px(Px(1.0));
                                            // new-york-v4: `-mx-1 h-px`.
                                            layout.margin.left =
                                                fret_ui::element::MarginEdge::Px(Px(-4.0));
                                            layout.margin.right =
                                                fret_ui::element::MarginEdge::Px(Px(-4.0));
                                            layout
                                        },
                                        background: Some(border),
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                );
                                sep = sep.attach_semantics(
                                    SemanticsDecoration::default().role(SemanticsRole::Separator),
                                );
                                if let Some(test_id) = test_id.clone() {
                                    sep = sep.test_id(test_id);
                                }
                                out.push(sep);
                            }
                            CommandPaletteRenderRow::Loading(loading) => {
                                out.push(loading.into_element(cx));
                            }
                            CommandPaletteRenderRow::Item(idx) => {
                                let Some(item) = items.get_mut(idx).and_then(Option::take) else {
                                    continue;
                                };

                                let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                                let focusable = tab_stop.is_some_and(|i| i == idx);

                                let query_for_row = query_for_render.clone();
                                let value_key = item.value.clone();
                                let value_for_select = item.value.clone();
                                let label = item.label.clone();
                                let test_id = item.test_id.clone();
                                let chrome_test_id = test_id
                                    .clone()
                                    .map(|id| Arc::<str>::from(format!("{id}.chrome")));
                                let command = item.command;
                                let on_select = item.on_select.clone();
                                let on_select_value = item.on_select_value.clone();
                                let children = item.children;
                                let leading_icon = item.leading_icon.clone();
                                let text_style = text_style.clone();

                                out.push(cx.keyed(value_key, move |cx| {
                                    cx.pressable(
                                        PressableProps {
                                            layout: item_layout,
                                            enabled,
                                            focusable,
                                            focus_ring: Some(ring),
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::ListBoxOption),
                                                label: Some(label.clone()),
                                                test_id: test_id.clone(),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, st| {
                                            cx.pressable_dispatch_command_if_enabled_opt(command);
                                            if on_select.is_some() || on_select_value.is_some() {
                                                let on_select = on_select.clone();
                                                let on_select_value = on_select_value.clone();
                                                let value = value_for_select.clone();
                                                cx.pressable_add_on_activate(Arc::new(
                                                    move |host, action_cx, reason| {
                                                        if let Some(on_select_value) =
                                                            on_select_value.clone()
                                                        {
                                                            on_select_value(
                                                                host,
                                                                action_cx,
                                                                reason,
                                                                value.clone(),
                                                            );
                                                        }
                                                        if let Some(on_select) = on_select.clone() {
                                                            on_select(host, action_cx, reason);
                                                        }
                                                    },
                                                ));
                                            }
                                            let hovered = st.hovered && !st.pressed;
                                            let pressed = st.pressed;
                                            let focused = st.focused;
                                            let selected = focused || hovered || pressed;

                                            let bg = selected.then_some(bg_hover);
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
                                                }
                                                .into(),
                                                background: bg,
                                                shadow: None,
                                                border: Edges::all(Px(0.0)),
                                                border_color: None,
                                                corner_radii: Corners::all(radius),
                                                ..Default::default()
                                            };

                                            let child = cx.container(props, move |cx| {
                                                let text_fg = if enabled {
                                                    if selected { fg_selected } else { fg }
                                                } else {
                                                    fg_disabled
                                                };
                                                let nonmatch_text_fg = if !enabled {
                                                    icon_fg_disabled
                                                } else if selected {
                                                    text_fg
                                                } else {
                                                    icon_fg
                                                };
                                                let effective_icon_fg =
                                                    if enabled { icon_fg } else { icon_fg_disabled };
	                                                current_color::scope_children(
	                                                    cx,
	                                                    ColorRef::Color(text_fg),
	                                                    |cx| {
	                                                        let dir =
	                                                            crate::use_direction(cx, None);
	                                                        let justify = crate::rtl::inline_start_end_pair(
	                                                            dir,
	                                                            MainAlign::Start,
	                                                            MainAlign::End,
	                                                        )
	                                                        .0;

	                                                        vec![cx.row(
	                                                            RowProps {
	                                                                layout: LayoutStyle::default(),
	                                                                gap: row_gap.into(),
	                                                                padding: Edges::all(Px(0.0))
	                                                                    .into(),
	                                                                justify,
	                                                                align: CrossAlign::Center,
	                                                            },
	                                                            move |cx| {
	                                                                if children.is_empty() {
	                                                                    let label_el =
	                                                                        cmdk_highlighted_label(
	                                                                            cx,
	                                                                            label.clone(),
	                                                                            query_for_row.as_ref(),
	                                                                            text_fg,
	                                                                            nonmatch_text_fg,
	                                                                            text_style.clone(),
	                                                                        );
                                                                    let icon_el = leading_icon
                                                                        .clone()
                                                                        .map(|icon| {
                                                                            decl_icon::icon_with(
                                                                                cx,
                                                                                icon,
                                                                                None,
                                                                                Some(
                                                                                    ColorRef::Color(
                                                                                        effective_icon_fg,
                                                                                    ),
                                                                                ),
                                                                            )
                                                                        });
                                                                    crate::rtl::vec_main_with_inline_start(
                                                                        dir,
                                                                        label_el,
                                                                        icon_el,
                                                                    )
                                                                } else {
                                                                    children
                                                                }
                                                            },
	                                                        )]
	                                                    },
	                                                )
	                                            });

                                            let mut chrome = child;
                                            if let Some(test_id) = chrome_test_id.clone() {
                                                chrome = chrome.test_id(test_id);
                                            }

                                            vec![chrome]
                                        },
                                    )
                                }));
                            }
                        }
                    }

                    out
                },
            )])
            .refine_layout(scroll)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ListBox)
                    .busy(list_busy),
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteA11ySelectedMode {
    /// Use `aria-selected` to reflect the currently highlighted/active row.
    Active,
    /// Use `aria-selected` to reflect the committed selection (`checked`) instead of highlight.
    ///
    /// This is a better fit for combobox/listbox conformance where active highlight is exposed via
    /// `active_descendant` on the input.
    Checked,
}

pub struct CommandPalette {
    model: Model<String>,
    entries: Vec<CommandEntry>,
    disabled: bool,
    should_filter: bool,
    filter: Option<CommandPaletteFilterFn>,
    value: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    auto_highlight: bool,
    wrap: bool,
    vim_bindings: bool,
    disable_pointer_selection: bool,
    empty_text: Arc<str>,
    a11y_label: Arc<str>,
    placeholder: Option<Arc<str>>,
    input_role: Option<SemanticsRole>,
    input_expanded: Option<bool>,
    input_test_id: Option<Arc<str>>,
    list_test_id: Option<Arc<str>>,
    list_multiselectable: bool,
    a11y_selected_mode: CommandPaletteA11ySelectedMode,
    on_value_change: Option<OnValueChange>,
    pending_dispatch: Option<Arc<std::sync::Mutex<Option<PendingCommandDispatch>>>>,
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
    test_id_input: Option<Arc<str>>,
    test_id_item_prefix: Option<Arc<str>>,
    test_id_heading_prefix: Option<Arc<str>>,
    pub(crate) input_id_out_cell: Option<Rc<Cell<Option<GlobalElementId>>>>,
    pub(crate) list_id_out_cell: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

enum CommandPaletteRenderRow {
    Heading(Arc<str>),
    GroupPad,
    Separator(Option<Arc<str>>),
    Loading(CommandLoading),
    Item(usize),
}

fn command_palette_render_rows_for_query_with_options(
    entries: Vec<CommandEntry>,
    query: &str,
    should_filter: bool,
    filter: Option<&CommandPaletteFilter>,
) -> (
    Vec<CommandPaletteRenderRow>,
    Vec<Option<CommandItem>>,
    Vec<Option<u32>>,
) {
    enum PendingRow {
        Heading(Arc<str>),
        Separator(CommandSeparator),
        Loading(CommandLoading),
        Item {
            group: Option<u32>,
            item: CommandItem,
        },
    }

    let has_search = !query.trim().is_empty();
    let query_for_filter = if should_filter { query } else { "" };
    let score_item = |item: &CommandItem| -> f32 {
        if query_for_filter.is_empty() {
            return 1.0;
        }

        let mut aliases: Vec<&str> = Vec::with_capacity(1 + item.keywords.len());
        if item.value.as_ref() != item.label.as_ref() {
            aliases.push(item.value.as_ref());
        }
        for kw in &item.keywords {
            aliases.push(kw.as_ref());
        }

        if let Some(filter) = filter {
            filter(item.label.as_ref(), query_for_filter, &aliases)
        } else {
            cmdk_score::command_score(item.label.as_ref(), query_for_filter, &aliases)
        }
    };

    let mut pending_rows: Vec<PendingRow> = Vec::new();
    let mut next_group_id: u32 = 0;
    if query_for_filter.is_empty() {
        for entry in entries {
            match entry {
                CommandEntry::Item(item) => {
                    let score = score_item(&item);
                    if score > 0.0 || item.force_mount {
                        pending_rows.push(PendingRow::Item { group: None, item });
                    }
                }
                CommandEntry::Separator(sep) => {
                    // cmdk: separators are visible when the search query is empty (or when
                    // `alwaysRender` is true), even when `shouldFilter` is false.
                    if !has_search || sep.always_render {
                        pending_rows.push(PendingRow::Separator(sep));
                    }
                }
                CommandEntry::Loading(loading) => {
                    pending_rows.push(PendingRow::Loading(loading));
                }
                CommandEntry::Group(group) => {
                    if group.items.is_empty() {
                        continue;
                    }

                    let group_id = next_group_id;
                    next_group_id = next_group_id.saturating_add(1);

                    if let Some(heading) = group.heading {
                        pending_rows.push(PendingRow::Heading(heading));
                    }
                    pending_rows.extend(group.items.into_iter().map(|item| PendingRow::Item {
                        group: Some(group_id),
                        item,
                    }));
                }
            }
        }
    } else {
        // cmdk sorts items globally by score and groups by highest item score. It also pushes
        // grouped items below ungrouped items.
        let mut root_items: Vec<(usize, f32, CommandItem)> = Vec::new();
        let mut groups: Vec<(
            usize,
            f32,
            Option<Arc<str>>,
            bool,
            Vec<(usize, f32, CommandItem)>,
        )> = Vec::new();
        let mut always_render_separators: Vec<(usize, CommandSeparator)> = Vec::new();
        let mut loading_rows: Vec<(usize, CommandLoading)> = Vec::new();

        for (entry_idx, entry) in entries.into_iter().enumerate() {
            match entry {
                CommandEntry::Item(item) => {
                    let score = score_item(&item);
                    if score > 0.0 || item.force_mount {
                        root_items.push((entry_idx, score, item));
                    }
                }
                CommandEntry::Separator(sep) => {
                    if sep.always_render {
                        always_render_separators.push((entry_idx, sep));
                    }
                }
                CommandEntry::Loading(loading) => {
                    loading_rows.push((entry_idx, loading));
                }
                CommandEntry::Group(group) => {
                    if group.items.is_empty() && !(group.force_mount && group.heading.is_some()) {
                        continue;
                    }

                    let mut scored_items: Vec<(usize, f32, CommandItem)> = group
                        .items
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, item)| {
                            let score = score_item(&item);
                            (score > 0.0 || item.force_mount).then_some((idx, score, item))
                        })
                        .collect();

                    if scored_items.is_empty() && !group.force_mount {
                        continue;
                    }

                    scored_items.sort_by(|(a_idx, a_score, _), (b_idx, b_score, _)| {
                        b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
                    });

                    let max_score = scored_items
                        .iter()
                        .map(|(_, score, _)| *score)
                        .fold(0.0_f32, f32::max);

                    if scored_items.is_empty() && group.heading.is_none() {
                        continue;
                    }

                    groups.push((
                        entry_idx,
                        max_score,
                        group.heading,
                        group.force_mount,
                        scored_items,
                    ));
                }
            }
        }

        root_items.sort_by(|(a_idx, a_score, _), (b_idx, b_score, _)| {
            b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
        });
        groups.sort_by(|(a_idx, a_score, _, _, _), (b_idx, b_score, _, _, _)| {
            b_score.total_cmp(a_score).then_with(|| a_idx.cmp(b_idx))
        });

        loading_rows.sort_by_key(|(idx, _)| *idx);
        always_render_separators.sort_by_key(|(idx, _)| *idx);

        pending_rows.extend(
            loading_rows
                .into_iter()
                .map(|(_, row)| PendingRow::Loading(row)),
        );
        pending_rows.extend(
            always_render_separators
                .into_iter()
                .map(|(_, sep)| PendingRow::Separator(sep)),
        );
        pending_rows.extend(
            root_items
                .into_iter()
                .map(|(_, _, item)| PendingRow::Item { group: None, item }),
        );
        for (_, _, heading, _force_mount, scored_items) in groups {
            let group_id = next_group_id;
            next_group_id = next_group_id.saturating_add(1);

            if let Some(heading) = heading {
                pending_rows.push(PendingRow::Heading(heading));
            }
            pending_rows.extend(
                scored_items
                    .into_iter()
                    .map(|(_, _, item)| PendingRow::Item {
                        group: Some(group_id),
                        item,
                    }),
            );
        }
    }

    let mut has_item_from: Vec<bool> = vec![false; pending_rows.len() + 1];
    for idx in (0..pending_rows.len()).rev() {
        has_item_from[idx] =
            has_item_from[idx + 1] || matches!(pending_rows[idx], PendingRow::Item { .. });
    }

    let mut filtered_rows: Vec<PendingRow> = Vec::with_capacity(pending_rows.len());
    let mut seen_item_before = false;
    let mut prev_is_sep = false;
    for (idx, row) in pending_rows.into_iter().enumerate() {
        match row {
            PendingRow::Separator(sep) => {
                if prev_is_sep {
                    continue;
                }
                if !sep.always_render && (!seen_item_before || !has_item_from[idx + 1]) {
                    continue;
                }
                prev_is_sep = true;
                filtered_rows.push(PendingRow::Separator(sep));
            }
            PendingRow::Loading(loading) => {
                prev_is_sep = false;
                filtered_rows.push(PendingRow::Loading(loading));
            }
            PendingRow::Item { group, item } => {
                seen_item_before = true;
                prev_is_sep = false;
                filtered_rows.push(PendingRow::Item { group, item });
            }
            PendingRow::Heading(h) => {
                prev_is_sep = false;
                filtered_rows.push(PendingRow::Heading(h));
            }
        }
    }

    let mut items: Vec<CommandItem> = Vec::new();
    let mut item_groups: Vec<Option<u32>> = Vec::new();
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
            PendingRow::Separator(sep) => {
                vec![CommandPaletteRenderRow::Separator(sep.test_id.clone())]
            }
            PendingRow::Loading(loading) => {
                vec![CommandPaletteRenderRow::Loading(loading)]
            }
            PendingRow::Item { group, item } => {
                let idx = items.len();
                items.push(item);
                item_groups.push(group);
                vec![CommandPaletteRenderRow::Item(idx)]
            }
        })
        .collect();

    let items = items.into_iter().map(Some).collect();
    (render_rows, items, item_groups)
}

impl std::fmt::Debug for CommandPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPalette")
            .field("entries_len", &self.entries.len())
            .field("disabled", &self.disabled)
            .field("should_filter", &self.should_filter)
            .field("filter", &self.filter.is_some())
            .field("value", &self.value.is_some())
            .field(
                "default_value",
                &self.default_value.as_ref().map(|s| s.as_ref()),
            )
            .field("on_value_change", &self.on_value_change.is_some())
            .field("wrap", &self.wrap)
            .field("vim_bindings", &self.vim_bindings)
            .field("disable_pointer_selection", &self.disable_pointer_selection)
            .field("empty_text", &self.empty_text.as_ref())
            .field("a11y_label", &self.a11y_label.as_ref())
            .field("input_role", &self.input_role)
            .field("input_expanded", &self.input_expanded)
            .field("on_value_change", &self.on_value_change.is_some())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("scroll", &self.scroll)
            .field(
                "test_id_input",
                &self.test_id_input.as_ref().map(|s| s.as_ref()),
            )
            .field(
                "test_id_item_prefix",
                &self.test_id_item_prefix.as_ref().map(|s| s.as_ref()),
            )
            .field(
                "test_id_heading_prefix",
                &self.test_id_heading_prefix.as_ref().map(|s| s.as_ref()),
            )
            .field("input_id_out_cell", &self.input_id_out_cell.is_some())
            .field("list_id_out_cell", &self.list_id_out_cell.is_some())
            .finish()
    }
}

impl CommandPalette {
    pub fn new(model: Model<String>, items: impl IntoIterator<Item = CommandItem>) -> Self {
        Self {
            model,
            entries: items.into_iter().map(CommandEntry::Item).collect(),
            disabled: false,
            // cmdk default: should filter/sort when search is non-empty.
            should_filter: true,
            filter: None,
            value: None,
            default_value: None,
            auto_highlight: true,
            // cmdk default: no loop unless explicitly enabled via `loop`.
            wrap: false,
            // cmdk default: ctrl+n/j/p/k keybinds enabled.
            vim_bindings: true,
            // cmdk default: pointer selection enabled unless explicitly disabled.
            disable_pointer_selection: false,
            empty_text: Arc::from("No results."),
            a11y_label: Arc::from("Command input"),
            placeholder: None,
            input_role: Some(SemanticsRole::ComboBox),
            input_expanded: None,
            input_test_id: None,
            list_test_id: None,
            list_multiselectable: false,
            a11y_selected_mode: CommandPaletteA11ySelectedMode::Active,
            on_value_change: None,
            pending_dispatch: None,
            input_wrapper_h: Px(36.0).into(),
            input_h: Px(40.0).into(),
            input_icon_size: Px(16.0).into(),
            item_pad_y: MetricRef::space(Space::N1p5),
            group_pad_x: MetricRef::space(Space::N1),
            group_pad_y: MetricRef::space(Space::N1),
            group_next_top_pad_zero: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            scroll: LayoutRefinement::default()
                .max_h(Px(300.0))
                .w_full()
                .min_w_0(),
            test_id_input: None,
            test_id_item_prefix: None,
            test_id_heading_prefix: None,
            input_id_out_cell: None,
            list_id_out_cell: None,
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
        items: impl IntoIterator<Item = CommandItem>,
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
        self.input_wrapper_h = Px(48.0).into();
        self.input_h = Px(48.0).into();
        self.input_icon_size = Px(20.0).into();
        self.item_pad_y = MetricRef::space(Space::N3);
        self.group_pad_x = MetricRef::space(Space::N2);
        self.group_pad_y = MetricRef::space(Space::N1);
        self.group_next_top_pad_zero = true;
        self
    }

    fn pending_dispatch(
        mut self,
        pending_dispatch: Arc<std::sync::Mutex<Option<PendingCommandDispatch>>>,
    ) -> Self {
        self.pending_dispatch = Some(pending_dispatch);
        self
    }

    /// cmdk: `shouldFilter` (default true). When false, the palette does not filter or sort items
    /// based on the query.
    pub fn should_filter(mut self, should_filter: bool) -> Self {
        self.should_filter = should_filter;
        self
    }

    /// cmdk: `filter`. Overrides the scoring function used for filtering/sorting when
    /// `should_filter` is true.
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&str, &str, &[&str]) -> f32 + Send + Sync + 'static,
    {
        self.filter = Some(Arc::new(filter));
        self
    }

    /// cmdk: `value` / `defaultValue` (selected item value).
    ///
    /// This keeps input focus on the combobox while the highlighted listbox option is tracked
    /// separately (cmdk-style active descendant).
    pub fn value(mut self, value: Option<Model<Option<Arc<str>>>>) -> Self {
        self.value = value;
        self
    }

    /// cmdk: `defaultValue` (initial selected item value).
    ///
    /// When omitted, the palette selects the first enabled item.
    pub fn default_value(mut self, default_value: impl Into<Arc<str>>) -> Self {
        self.default_value = Some(cmdk_trimmed_arc(default_value.into()));
        self
    }

    /// When enabled, highlights the first enabled option when nothing has been explicitly
    /// navigated to yet (cmdk default behavior).
    ///
    /// Base UI Combobox defaults to `false` and opts in via `autoHighlight`.
    pub fn auto_highlight(mut self, auto_highlight: bool) -> Self {
        self.auto_highlight = auto_highlight;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// cmdk: `onValueChange(value)`. Called when the palette's active (highlighted) `value` changes.
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// cmdk: `loop`. When `true`, selection wraps at the list boundaries.
    pub fn loop_(mut self, loop_: bool) -> Self {
        self.wrap = loop_;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Enables cmdk-style "vim bindings" (Ctrl+N/J/P/K) for selection navigation.
    pub fn vim_bindings(mut self, vim_bindings: bool) -> Self {
        self.vim_bindings = vim_bindings;
        self
    }

    /// Disables cmdk-style pointer-based highlight changes (hover does not move the active item).
    pub fn disable_pointer_selection(mut self, disable_pointer_selection: bool) -> Self {
        self.disable_pointer_selection = disable_pointer_selection;
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = CommandEntry>) -> Self {
        self.entries = entries.into_iter().collect();
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

    pub fn input_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.input_test_id = Some(test_id.into());
        self
    }

    pub fn list_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.list_test_id = Some(test_id.into());
        self
    }

    pub fn list_multiselectable(mut self, multiselectable: bool) -> Self {
        self.list_multiselectable = multiselectable;
        self
    }

    pub fn a11y_selected_mode(mut self, mode: CommandPaletteA11ySelectedMode) -> Self {
        self.a11y_selected_mode = mode;
        self
    }

    /// Installs a stable `test_id` on the command input (for automation).
    pub fn test_id_input(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_input = Some(id.into());
        self
    }

    /// Installs a shared stable `test_id` prefix for command palette surfaces.
    ///
    /// This derives:
    /// - `{prefix}-input`
    /// - `{prefix}-listbox`
    /// - `{prefix}-item-{sanitized_value}`
    /// - `{prefix}-heading-{sanitized_heading}`
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        let prefix = prefix.into();
        self.input_test_id = Some(Arc::<str>::from(format!("{prefix}-input")));
        self.list_test_id = Some(Arc::<str>::from(format!("{prefix}-listbox")));
        self.test_id_input = Some(Arc::<str>::from(format!("{prefix}-input")));
        self.test_id_item_prefix = Some(Arc::<str>::from(format!("{prefix}-item-")));
        self.test_id_heading_prefix = Some(Arc::<str>::from(format!("{prefix}-heading-")));
        self
    }

    /// Installs stable `test_id`s on item rows using `{prefix}{sanitized_value}`.
    pub fn test_id_item_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_item_prefix = Some(prefix.into());
        self
    }

    /// Installs stable `test_id`s on group headings using `{prefix}{sanitized_heading}`.
    pub fn test_id_heading_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_heading_prefix = Some(prefix.into());
        self
    }

    pub(crate) fn input_id_out_cell(mut self, cell: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.input_id_out_cell = Some(cell);
        self
    }

    pub(crate) fn list_id_out_cell(mut self, cell: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.list_id_out_cell = Some(cell);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Clone)]
        struct PaletteEntry {
            value: Arc<str>,
            command: Option<CommandId>,
            on_select: Option<fret_ui::action::OnActivate>,
            on_select_value: Option<OnSelectValueAction>,
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
            vim_bindings: Rc<Cell<bool>>,
            entries: Rc<RefCell<Arc<[PaletteEntry]>>>,
            item_groups: Rc<RefCell<Arc<[Option<u32>]>>>,
            handler: fret_ui::action::OnKeyDown,
        }

        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let input_wrapper_h_fallback = self.input_wrapper_h.resolve(&theme);
            let input_h_fallback = self.input_h.resolve(&theme);
            let input_icon_size_fallback = self.input_icon_size.resolve(&theme);
            let item_pad_y = self.item_pad_y.resolve(&theme);
            let group_pad_x = self.group_pad_x.resolve(&theme);
            let group_pad_y = self.group_pad_y.resolve(&theme);
            let group_next_top_pad_zero = self.group_next_top_pad_zero;

            let disabled = self.disabled;
            let should_filter = self.should_filter;
            let filter = self.filter.clone();
            let value = self.value.clone();
            let default_value = self.default_value.clone();
            let wrap = self.wrap;
            let vim_bindings = self.vim_bindings;
            let disable_pointer_selection = self.disable_pointer_selection;
            let input_test_id = self.input_test_id.clone();
            let list_test_id = self.list_test_id.clone();
            let list_multiselectable = self.list_multiselectable;
            let list_id_out_cell = self.list_id_out_cell.clone();
            let a11y_selected_mode = self.a11y_selected_mode;
            let on_value_change = self.on_value_change.clone();
            let pending_dispatch = self.pending_dispatch.clone();
            let test_id_input = self.test_id_input;
            let test_id_item_prefix = self.test_id_item_prefix;
            let test_id_heading_prefix = self.test_id_heading_prefix;
            let query = cx
                .watch_model(&self.model)
                .layout()
                .read_ref(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();
            let query_for_render: Arc<str> = Arc::from(query.as_str());

            let (render_rows, mut items, item_groups) =
                command_palette_render_rows_for_query_with_options(
                    self.entries,
                    query.as_str(),
                    should_filter,
                    filter.as_deref(),
                );
            let list_busy = render_rows
                .iter()
                .any(|row| matches!(row, CommandPaletteRenderRow::Loading(_)));

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
                        CommandPaletteRenderRow::Separator(_) => {
                            "separator".hash(&mut hasher);
                        }
                        CommandPaletteRenderRow::Loading(loading) => {
                            "loading".hash(&mut hasher);
                            loading.text.as_ref().hash(&mut hasher);
                            loading
                                .test_id
                                .as_ref()
                                .map(|s| s.as_ref())
                                .unwrap_or("")
                                .hash(&mut hasher);
                        }
                        CommandPaletteRenderRow::Item(idx) => {
                            "item".hash(&mut hasher);
                            if let Some(item) = items.get(*idx).and_then(|item| item.as_ref()) {
                                item.label.as_ref().hash(&mut hasher);
                                item.value.as_ref().hash(&mut hasher);
                                item_groups
                                    .get(*idx)
                                    .copied()
                                    .flatten()
                                    .unwrap_or(u32::MAX)
                                    .hash(&mut hasher);
                                item.keywords.len().hash(&mut hasher);
                                for kw in &item.keywords {
                                    (&**kw).hash(&mut hasher);
                                }
                                item.shortcut.as_deref().unwrap_or("").hash(&mut hasher);
                                item.disabled.hash(&mut hasher);
                                item.command
                                    .as_ref()
                                    .map(CommandId::as_str)
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
                .map(|item| {
                    let Some(item) = item.as_ref() else {
                        return (
                            PaletteEntry {
                                value: Arc::from(""),
                                command: None,
                                on_select: None,
                                on_select_value: None,
                                disabled: true,
                            },
                            true,
                        );
                    };
                    let disabled = disabled
                        || item.disabled
                        || (item.command.is_none()
                            && item.on_select.is_none()
                            && item.on_select_value.is_none());
                    (
                        PaletteEntry {
                            value: item.value.clone(),
                            command: item.command.clone(),
                            on_select: item.on_select.clone(),
                            on_select_value: item.on_select_value.clone(),
                            disabled,
                        },
                        disabled,
                    )
                })
                .unzip();
            let entries_arc: Arc<[PaletteEntry]> = Arc::from(entries.into_boxed_slice());

            let default_value_for_hook = default_value.clone();
            let active = controllable_state::use_controllable_model(cx, value, move || {
                default_value_for_hook.clone().map(cmdk_trimmed_arc)
            })
            .model();

            let _items_changed = cx.with_state(CommandPaletteState::default, |st| {
                if st.items_fingerprint != items_fingerprint {
                    st.items_fingerprint = items_fingerprint;
                    true
                } else {
                    false
                }
            });

            let auto_highlight = self.auto_highlight;
            let cur_active_raw = cx.watch_model(&active).cloned().unwrap_or(None);
            let cur_active = cur_active_raw.clone().map(cmdk_trimmed_arc);
            let next_active = if auto_highlight {
                cur_active
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
                    })
            } else {
                cur_active.as_ref().and_then(|v| {
                    entries_arc
                        .iter()
                        .enumerate()
                        .find(|(idx, e)| {
                            disabled_flags.get(*idx).copied() == Some(false)
                                && e.value.as_ref() == v.as_ref()
                        })
                        .map(|(_, e)| e.value.clone())
                })
            };
            if next_active != cur_active_raw {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&active, |v| *v = next_active.clone());
            }

            if let Some(handler) = on_value_change.as_ref() {
                let change = cx
                    .with_state(CommandPaletteValueChangeCallbackState::default, |state| {
                        command_palette_value_change_event(state, next_active.clone())
                    });
                if let Some(value) = change {
                    handler(value);
                }
            }

            let mut row_ids: Vec<fret_ui::elements::GlobalElementId> =
                Vec::with_capacity(items.len());

            let row_h = MetricRef::space(Space::N8).resolve(&theme);
            let row_gap = MetricRef::space(Space::N2).resolve(&theme);
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            let pad_y = item_pad_y;
            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);

            let bg_hover = item_bg_hover(&theme);
            let bg_selected = bg_hover;
            let fg_selected = theme.color_token("accent-foreground");
            let fg = theme.color_token("foreground");
            let fg_disabled = alpha_mul(fg, 0.5);
            let muted_fg = theme.color_token("muted-foreground");
            let muted_fg_disabled = alpha_mul(muted_fg, 0.5);
            let text_style = item_text_style(&theme);
            let item_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_h(row_h).min_w_0(),
            );

            let mut key_counts: HashMap<RowKey, u32> = HashMap::new();

            let active_idx = next_active.as_ref().and_then(|active_value| {
                items.iter().enumerate().find_map(|(idx, item)| {
                    let enabled = disabled_flags.get(idx).copied() == Some(false);
                    let Some(item) = item.as_ref() else {
                        return None;
                    };
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
                        let fg = theme.color_token("muted-foreground");
                        let style = heading_text_style(&theme);
                        let test_id_for_row = test_id_heading_prefix.clone().map(|prefix| {
                            let seg = sanitize_test_id_segment(heading.as_ref());
                            Arc::<str>::from(format!("{prefix}{seg}"))
                        });
                        let mut heading_row = cx.container(
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
                                }
                                .into(),
                                ..Default::default()
                            },
                            move |cx| {
                                let mut text = ui::text( heading)
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .nowrap()
                                    .text_color(ColorRef::Color(fg));

                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height).line_height_policy(
                                        fret_core::TextLineHeightPolicy::FixedFromStyle,
                                    );
                                }

                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }

                                vec![text.into_element(cx)]
                            },
                        );
                        if let Some(test_id) = test_id_for_row {
                            heading_row = heading_row.test_id(test_id);
                        }
                        heading_row
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
                    CommandPaletteRenderRow::Separator(test_id) => {
                        let border = border(&theme);
                        let mut sep = cx.container(
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
                        );
                        sep = sep.attach_semantics(
                            SemanticsDecoration::default().role(SemanticsRole::Separator),
                        );
                        if let Some(test_id) = test_id.clone() {
                            sep = sep.test_id(test_id);
                        }
                        sep
                    }
                    CommandPaletteRenderRow::Loading(loading) => loading.into_element(cx),
                    CommandPaletteRenderRow::Item(idx) => {
                        let Some(item) = items.get_mut(idx).and_then(Option::take) else {
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
                        let pending_dispatch_for_row = pending_dispatch.clone();
                        cx.keyed((base, occ), |cx| {
                            let enabled = disabled_flags.get(idx).copied() == Some(false);
                            let active_row = active_idx.is_some_and(|i| i == idx);

                            let label = item.label.clone();
                            let value = item.value.clone();
                            let checked = item.checked;
                            let show_checkmark = item.show_checkmark;
                            let test_id = item.test_id.clone();
                            let shortcut = item.shortcut.clone();
                            let command = item.command;
                            let on_select = item.on_select.clone();
                            let on_select_value = item.on_select_value.clone();
                            let leading_icon = item.leading_icon.clone();
                            let children = item.children;
                            let text_style = text_style.clone();

                            let selected_a11y = match a11y_selected_mode {
                                CommandPaletteA11ySelectedMode::Active => active_row,
                                CommandPaletteA11ySelectedMode::Checked => checked,
                            };

                            let test_id_for_row = test_id_item_prefix.clone().map(|prefix| {
                                let seg = sanitize_test_id_segment(value.as_ref());
                                let id = if occ == 0 {
                                    format!("{}{}", prefix, seg)
                                } else {
                                    format!("{}{}-{}", prefix, seg, occ)
                                };
                                Arc::<str>::from(id)
                            });
                            let chrome_test_id = test_id_for_row
                                .clone()
                                .or_else(|| test_id.clone())
                                .map(|id| Arc::<str>::from(format!("{id}.chrome")));

                            let mut row = cx.pressable(
                                PressableProps {
                                    layout: item_layout,
                                    enabled,
                                    focusable: false,
                                    focus_ring: None,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::ListBoxOption),
                                        label: Some(label.clone()),
                                        test_id: test_id.clone(),
                                        selected: selected_a11y,
                                        ..Default::default()
                                    }
                                    .with_collection_position(idx, item_count),
                                    ..Default::default()
                                },
                                move |cx, st| {
                                    if enabled
                                        && let Some(command) = command.clone()
                                        && let Some(pending_dispatch) =
                                            pending_dispatch_for_row.clone()
                                    {
                                        cx.pressable_add_on_activate(Arc::new({
                                            let command = command.clone();
                                            move |host, action_cx, reason| {
                                                if let Ok(mut slot) = pending_dispatch.lock() {
                                                    *slot = Some(PendingCommandDispatch {
                                                        command: command.clone(),
                                                        reason,
                                                    });
                                                }
                                                host.request_redraw(action_cx.window);
                                            }
                                        }));
                                    } else {
                                        cx.pressable_dispatch_command_if_enabled_opt(command);
                                    }
                                    if on_select.is_some() || on_select_value.is_some() {
                                        let on_select = on_select.clone();
                                        let on_select_value = on_select_value.clone();
                                        let value = value.clone();
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, action_cx, reason| {
                                                if let Some(on_select_value) =
                                                    on_select_value.clone()
                                                {
                                                    on_select_value(
                                                        host,
                                                        action_cx,
                                                        reason,
                                                        value.clone(),
                                                    );
                                                }
                                                if let Some(on_select) = on_select.clone() {
                                                    on_select(host, action_cx, reason);
                                                }
                                            },
                                        ));
                                    }
                                    if enabled && !disable_pointer_selection {
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
                                    let bg = if active_row {
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
                                        }
                                        .into(),
                                        background: bg,
                                        shadow: None,
                                        border: Edges::all(Px(0.0)),
                                        border_color: None,
                                        corner_radii: Corners::all(radius),
                                        ..Default::default()
                                    };

                                    let child = cx.container(props, move |cx| {
                                        let text_fg = if enabled {
                                            if active_row { fg_selected } else { fg }
                                        } else {
                                            fg_disabled
                                        };
                                        let nonmatch_text_fg = if !enabled {
                                            muted_fg_disabled
                                        } else if active_row {
                                            text_fg
                                        } else {
                                            muted_fg
                                        };
                                        let icon_fg =
                                            if enabled { muted_fg } else { muted_fg_disabled };
                                        current_color::scope_children(
                                            cx,
                                            ColorRef::Color(text_fg),
                                            |cx| {
                                                let dir = crate::use_direction(cx, None);
                                                vec![cx.row(
                                                    RowProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.width = Length::Fill;
                                                            layout
                                                        },
                                                        gap: row_gap.into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Center,
                                                    },
                                                    move |cx| {
                                                        if !children.is_empty() {
                                                            return children;
                                                        }

                                                        let left_justify = crate::rtl::inline_start_end_pair(
                                                            dir,
                                                            MainAlign::Start,
                                                            MainAlign::End,
                                                        )
                                                        .0;
                                                        let left = cx.row(
                                                            RowProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width =
                                                                        Length::Fill;
                                                                    layout.size.min_width =
                                                                        Some(Length::Px(Px(0.0)));
                                                                    layout.flex.grow = 1.0;
                                                                    layout.flex.shrink = 1.0;
                                                                    layout.flex.basis =
                                                                        Length::Px(Px(0.0));
                                                                    layout
                                                                },
                                                                gap: row_gap.into(),
                                                                padding: Edges::all(Px(0.0)).into(),
                                                                justify: left_justify,
                                                                align: CrossAlign::Center,
                                                            },
                                                            move |cx| {
                                                                let mut out: Vec<AnyElement> =
                                                                    Vec::with_capacity(
                                                                        usize::from(show_checkmark)
                                                                            + usize::from(
                                                                                leading_icon
                                                                                    .is_some(),
                                                                            )
                                                                            + 1,
                                                                    );

                                                                let label_el =
                                                                    cmdk_highlighted_label(
                                                                        cx,
                                                                        label.clone(),
                                                                        query_for_row.as_ref(),
                                                                        text_fg,
                                                                        nonmatch_text_fg,
                                                                        text_style.clone(),
                                                                    );

                                                                let icon_el = leading_icon
                                                                    .clone()
                                                                    .map(|icon| {
                                                                        decl_icon::icon_with(
                                                                            cx,
                                                                            icon,
                                                                            None,
                                                                            Some(ColorRef::Color(
                                                                                icon_fg,
                                                                            )),
                                                                        )
                                                                    });

                                                                let check_el =
                                                                    show_checkmark.then(|| {
                                                                        let icon =
                                                                            decl_icon::icon_with(
                                                                                cx,
                                                                                ids::ui::CHECK,
                                                                                Some(Px(16.0)),
                                                                                Some(
                                                                                    ColorRef::Color(
                                                                                        icon_fg,
                                                                                    ),
                                                                                ),
                                                                            );
                                                                        cx.opacity(
                                                                            if checked {
                                                                                1.0
                                                                            } else {
                                                                                0.0
                                                                            },
                                                                            move |_cx| vec![icon],
                                                                        )
                                                                    });

                                                                let mut prefix = Vec::new();
                                                                if let Some(check_el) = check_el {
                                                                    prefix.push(check_el);
                                                                }
                                                                if let Some(icon_el) = icon_el {
                                                                    prefix.push(icon_el);
                                                                }
                                                                out.extend(
                                                                    crate::rtl::concat_main_with_inline_start_vec(
                                                                        dir, label_el, prefix,
                                                                    ),
                                                                );

                                                                out
                                                            },
                                                        );

                                                        if let Some(shortcut) = shortcut.clone() {
                                                            let shortcut =
                                                                CommandShortcut::new(shortcut)
                                                                    .inline()
                                                                    .into_element(cx);
                                                            let (a, b) =
                                                                crate::rtl::inline_start_end_pair(
                                                                    dir, left, shortcut,
                                                                );
                                                            vec![a, b]
                                                        } else {
                                                            vec![left]
                                                        }
                                                    },
                                                )]
                                            },
                                        )
                                    });

                                    let mut chrome = child;
                                    if let Some(test_id) = chrome_test_id.clone() {
                                        chrome = chrome.test_id(test_id);
                                    }

                                    vec![chrome]
                                },
                            );

                            if let Some(test_id) = test_id_for_row {
                                row = row.attach_semantics(
                                    SemanticsDecoration::default().test_id(test_id),
                                );
                            }

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
            }
            .into();

            let a11y_label = self.a11y_label.clone();
            let effective_input_test_id = input_test_id.clone().or_else(|| test_id_input.clone());
            let mut input = command_text_input(
                cx,
                self.model.clone(),
                a11y_label,
                self.placeholder.clone(),
                self.input_role,
                effective_input_test_id.clone(),
                active_descendant,
                self.input_expanded,
                MetricRef::space(Space::N3).resolve(&theme),
                Length::Px(input_h),
            );
            if let Some(test_id) = test_id_input.clone() {
                input = input.attach_semantics(SemanticsDecoration::default().test_id(test_id));
            }
            let input_id = input.id;
            if let Some(cell) = self.input_id_out_cell.clone() {
                cell.set(Some(input_id));
            }

            let icon_fg = theme.color_token("muted-foreground");
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::SEARCH,
                Some(icon_size),
                Some(ColorRef::Color(icon_fg)),
            );
            let icon = cx.opacity(0.5, move |_cx| vec![icon]);
            let dir = crate::use_direction(cx, None);

            let mut input = cx.row(
                RowProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    gap: gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                },
                move |_cx| {
                    let (a, b) = rtl::inline_start_end_pair(dir, icon, input);
                    vec![a, b]
                },
            );
            let list_labelled_by = Some(input_id.0);

            let key_handler = cx.with_state(
                || {
                    let entries_cell: Rc<RefCell<Arc<[PaletteEntry]>>> =
                        Rc::new(RefCell::new(Arc::from([])));
                    let entries_read = entries_cell.clone();
                    let item_groups_cell: Rc<RefCell<Arc<[Option<u32>]>>> =
                        Rc::new(RefCell::new(Arc::from([])));
                    let item_groups_read = item_groups_cell.clone();
                    let disabled_cell = Rc::new(Cell::new(false));
                    let wrap_cell = Rc::new(Cell::new(true));
                    let vim_bindings_cell = Rc::new(Cell::new(true));

                    let disabled_read = disabled_cell.clone();
                    let wrap_read = wrap_cell.clone();
                    let vim_bindings_read = vim_bindings_cell.clone();

                    let handler: fret_ui::action::OnKeyDown =
                        Arc::new(move |host, action_cx, down| {
                            if disabled_read.get() {
                                return false;
                            }

                            // Align cmdk: avoid triggering navigation/activation shortcuts while IME composition
                            // is still active (e.g. CJK preedit). See `cmdk`'s `isComposing` guard.
                            if down.ime_composing {
                                return false;
                            }

                            let entries = entries_read.borrow();
                            let disabled_flags: Vec<bool> =
                                entries.iter().map(|e| e.disabled).collect();
                            let groups = item_groups_read.borrow();

                            let current_value =
                                host.models_mut().get_cloned(&active).unwrap_or(None);
                            let current_idx = current_value.as_ref().and_then(|v| {
                                entries.iter().position(|e| e.value.as_ref() == v.as_ref())
                            });

                            let mut set_active_by_idx = |idx: Option<usize>| {
                                let next =
                                    idx.and_then(|i| entries.get(i)).map(|e| e.value.clone());
                                if next != current_value {
                                    let _ =
                                        host.models_mut().update(&active, |v| *v = next.clone());
                                    host.request_redraw(action_cx.window);
                                }
                            };

                            let move_by_item = |forward: bool,
                                                set_active_by_idx: &mut dyn FnMut(
                                Option<usize>,
                            )| {
                                let next_idx = cmdk_selection::next_active_index(
                                    &disabled_flags,
                                    current_idx,
                                    forward,
                                    wrap_read.get(),
                                );
                                set_active_by_idx(next_idx);
                            };

                            let move_to_edge = |forward: bool,
                                                set_active_by_idx: &mut dyn FnMut(
                                Option<usize>,
                            )| {
                                let next_idx = if forward {
                                    cmdk_selection::last_enabled(&disabled_flags)
                                } else {
                                    cmdk_selection::first_enabled(&disabled_flags)
                                };
                                set_active_by_idx(next_idx);
                            };

                            let move_by_group = |forward: bool,
                                                 set_active_by_idx: &mut dyn FnMut(
                                Option<usize>,
                            )| {
                                let Some(cur_idx) = current_idx else {
                                    move_by_item(forward, set_active_by_idx);
                                    return;
                                };
                                let Some(cur_group) = groups.get(cur_idx).copied().flatten() else {
                                    move_by_item(forward, set_active_by_idx);
                                    return;
                                };

                                // Group order is the order of appearance in the rendered rows.
                                let mut order: Vec<u32> = Vec::new();
                                let mut last: Option<u32> = None;
                                for g in groups.iter().copied().flatten() {
                                    if last != Some(g) {
                                        order.push(g);
                                        last = Some(g);
                                    }
                                }

                                let Some(pos) = order.iter().position(|g| *g == cur_group) else {
                                    move_by_item(forward, set_active_by_idx);
                                    return;
                                };

                                let candidates: Box<dyn Iterator<Item = u32>> = if forward {
                                    Box::new(order.iter().copied().skip(pos + 1))
                                } else {
                                    Box::new(order.iter().copied().take(pos).rev())
                                };

                                for group_id in candidates {
                                    let next_idx =
                                        groups.iter().enumerate().find_map(|(idx, g)| {
                                            (disabled_flags.get(idx).copied() == Some(false)
                                                && *g == Some(group_id))
                                            .then_some(idx)
                                        });
                                    if next_idx.is_some() {
                                        set_active_by_idx(next_idx);
                                        return;
                                    }
                                }

                                // Fallback to item-wise movement when no other enabled group exists.
                                move_by_item(forward, set_active_by_idx);
                            };

                            match down.key {
                                KeyCode::KeyN | KeyCode::KeyJ => {
                                    if vim_bindings_read.get() && down.modifiers.ctrl {
                                        move_by_item(true, &mut set_active_by_idx);
                                        return true;
                                    }
                                    false
                                }
                                KeyCode::KeyP | KeyCode::KeyK => {
                                    if vim_bindings_read.get() && down.modifiers.ctrl {
                                        move_by_item(false, &mut set_active_by_idx);
                                        return true;
                                    }
                                    false
                                }
                                KeyCode::ArrowDown | KeyCode::ArrowUp => {
                                    let forward = down.key == KeyCode::ArrowDown;
                                    if down.modifiers.meta {
                                        move_to_edge(forward, &mut set_active_by_idx);
                                    } else if down.modifiers.alt {
                                        move_by_group(forward, &mut set_active_by_idx);
                                    } else {
                                        move_by_item(forward, &mut set_active_by_idx);
                                    }
                                    true
                                }
                                KeyCode::Home => {
                                    let next_idx = cmdk_selection::first_enabled(&disabled_flags);
                                    set_active_by_idx(next_idx);
                                    true
                                }
                                KeyCode::End => {
                                    let next_idx = cmdk_selection::last_enabled(&disabled_flags);
                                    set_active_by_idx(next_idx);
                                    true
                                }
                                KeyCode::PageDown | KeyCode::PageUp => {
                                    let forward = down.key == KeyCode::PageDown;
                                    let next_idx = cmdk_selection::advance_active_index(
                                        &disabled_flags,
                                        current_idx,
                                        forward,
                                        wrap_read.get(),
                                        10,
                                    );
                                    set_active_by_idx(next_idx);
                                    true
                                }
                                KeyCode::Enter | KeyCode::NumpadEnter => {
                                    let Some(idx) = cmdk_selection::clamp_active_index(
                                        &disabled_flags,
                                        current_idx,
                                    ) else {
                                        return false;
                                    };

                                    let Some(entry) = entries.get(idx) else {
                                        return false;
                                    };

                                    if let Some(on_select_value) = entry.on_select_value.clone() {
                                        on_select_value(
                                            host,
                                            action_cx,
                                            ActivateReason::Keyboard,
                                            entry.value.clone(),
                                        );
                                    }

                                    if let Some(on_select) = entry.on_select.clone() {
                                        on_select(host, action_cx, ActivateReason::Keyboard);
                                    }

                                    if let Some(command) = entry.command.clone() {
                                        if let Some(pending_dispatch) = pending_dispatch.clone() {
                                            if let Ok(mut slot) = pending_dispatch.lock() {
                                                *slot = Some(PendingCommandDispatch {
                                                    command,
                                                    reason: ActivateReason::Keyboard,
                                                });
                                            }
                                        } else {
                                            host.dispatch_command(Some(action_cx.window), command);
                                        }
                                    }
                                    true
                                }
                                _ => false,
                            }
                        });

                    KeyHandlerState {
                        disabled: disabled_cell,
                        wrap: wrap_cell,
                        vim_bindings: vim_bindings_cell,
                        entries: entries_cell,
                        item_groups: item_groups_cell,
                        handler,
                    }
                },
                |state: &mut KeyHandlerState| {
                    state.disabled.set(disabled);
                    state.wrap.set(wrap);
                    state.vim_bindings.set(vim_bindings);
                    *state.entries.borrow_mut() = entries_arc.clone();
                    *state.item_groups.borrow_mut() =
                        Arc::from(item_groups.clone().into_boxed_slice());
                    state.handler.clone()
                },
            );

            cx.key_on_key_down_for(input_id, key_handler);

            if disabled {
                input = cx.opacity(0.5, move |_cx| vec![input]);
                input = input.attach_semantics(SemanticsDecoration {
                    role: Some(SemanticsRole::Generic),
                    disabled: Some(true),
                    ..Default::default()
                });
            }

            let scroll_layout = self.scroll.w_full().min_w_0();
            let list = if rows.is_empty() {
                let empty = self.empty_text;
                CommandEmpty::new(empty).into_element(cx)
            } else {
                let scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());
                let scroll_area = ScrollArea::new(vec![
                    cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.min_height = Some(Length::Px(Px(0.0)));
                                layout
                            },
                            direction: fret_core::Axis::Vertical,
                            gap: Px(0.0).into(),
                            padding: Edges {
                                top: group_pad_y,
                                right: group_pad_x,
                                bottom: group_pad_y,
                                left: group_pad_x,
                            }
                            .into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                            ..Default::default()
                        },
                        move |_cx| rows,
                    ),
                ])
                .scroll_handle(scroll_handle.clone())
                .refine_layout(scroll_layout.clone())
                .into_element(cx);

                if let Some(active_row_element) = active_row_element {
                    let _ = active_desc::scroll_active_element_into_view_y(
                        cx,
                        &scroll_handle,
                        scroll_area.id,
                        active_row_element,
                    );
                }

                scroll_area
            };

            if let Some(cell) = list_id_out_cell.as_ref() {
                cell.set(Some(list.id));
            }

            let list = list.attach_semantics(SemanticsDecoration {
                role: Some(SemanticsRole::ListBox),
                busy: Some(list_busy),
                multiselectable: list_multiselectable.then_some(true),
                labelled_by_element: list_labelled_by,
                ..Default::default()
            });
            let list = if let Some(test_id) = list_test_id.clone() {
                list.test_id(test_id)
            } else {
                list
            };

            Command::new(vec![cx.container(wrapper, move |_cx| vec![input]), list])
                .refine_style(self.chrome)
                .refine_layout(self.layout)
                .into_element(cx)
        })
    }
}

pub struct CommandDialog {
    open: Model<bool>,
    query: Model<String>,
    entries: Vec<CommandEntry>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    should_filter: bool,
    filter: Option<CommandPaletteFilterFn>,
    value: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    on_value_change: Option<OnValueChange>,
    wrap: bool,
    vim_bindings: bool,
    disable_pointer_selection: bool,
    close_on_select: bool,
    empty_text: Arc<str>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for CommandDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDialog")
            .field("open", &"<model>")
            .field("query", &"<model>")
            .field("entries_len", &self.entries.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("disabled", &self.disabled)
            .field("should_filter", &self.should_filter)
            .field("filter", &self.filter.is_some())
            .field("value", &self.value.is_some())
            .field(
                "default_value",
                &self.default_value.as_ref().map(|s| s.as_ref()),
            )
            .field("wrap", &self.wrap)
            .field("vim_bindings", &self.vim_bindings)
            .field("disable_pointer_selection", &self.disable_pointer_selection)
            .field("close_on_select", &self.close_on_select)
            .field("empty_text", &self.empty_text.as_ref())
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_with_reason",
                &self.on_open_change_with_reason.is_some(),
            )
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .finish()
    }
}

impl CommandDialog {
    pub fn new(
        open: Model<bool>,
        query: Model<String>,
        items: impl IntoIterator<Item = CommandItem>,
    ) -> Self {
        Self {
            open,
            query,
            entries: items.into_iter().map(CommandEntry::Item).collect(),
            a11y_label: None,
            disabled: false,
            should_filter: true,
            filter: None,
            value: None,
            default_value: None,
            on_value_change: None,
            wrap: false,
            vim_bindings: true,
            disable_pointer_selection: false,
            close_on_select: true,
            empty_text: Arc::from("No results."),
            on_open_change: None,
            on_open_change_with_reason: None,
            on_open_change_complete: None,
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
        items: impl IntoIterator<Item = CommandItem>,
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
            should_filter: true,
            filter: None,
            value: None,
            default_value: None,
            on_value_change: None,
            wrap: false,
            vim_bindings: true,
            disable_pointer_selection: false,
            close_on_select: true,
            empty_text: Arc::from("No results."),
            on_open_change: None,
            on_open_change_with_reason: None,
            on_open_change_complete: None,
        }
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = CommandEntry>) -> Self {
        self.entries = entries.into_iter().collect();
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

    /// cmdk: `loop`. When `true`, selection wraps at the list boundaries.
    pub fn loop_(mut self, loop_: bool) -> Self {
        self.wrap = loop_;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Enables cmdk-style "vim bindings" (Ctrl+N/J/P/K) for selection navigation.
    pub fn vim_bindings(mut self, vim_bindings: bool) -> Self {
        self.vim_bindings = vim_bindings;
        self
    }

    /// Disables cmdk-style pointer-based highlight changes (hover does not move the active item).
    pub fn disable_pointer_selection(mut self, disable_pointer_selection: bool) -> Self {
        self.disable_pointer_selection = disable_pointer_selection;
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

    /// Called when the dialog open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
        self
    }

    /// Called when the dialog open state changes with reason metadata.
    pub fn on_open_change_with_reason(
        mut self,
        on_open_change_with_reason: Option<OnOpenChangeWithReason>,
    ) -> Self {
        self.on_open_change_with_reason = on_open_change_with_reason;
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.on_open_change_complete = on_open_change_complete;
        self
    }

    /// cmdk: `shouldFilter` (default true). When false, the dialog's internal palette does not
    /// filter or sort items based on the query.
    pub fn should_filter(mut self, should_filter: bool) -> Self {
        self.should_filter = should_filter;
        self
    }

    /// cmdk: `filter`. Overrides the scoring function used for filtering/sorting when
    /// `should_filter` is true.
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&str, &str, &[&str]) -> f32 + Send + Sync + 'static,
    {
        self.filter = Some(Arc::new(filter));
        self
    }

    /// cmdk: `value` / `defaultValue` (selected item value).
    pub fn value(mut self, value: Option<Model<Option<Arc<str>>>>) -> Self {
        self.value = value;
        self
    }

    /// cmdk: `onValueChange(value)`. Called when the dialog's internal palette changes its active
    /// (highlighted) `value`.
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// cmdk: `defaultValue` (initial selected item value).
    pub fn default_value(mut self, default_value: impl Into<Arc<str>>) -> Self {
        self.default_value = Some(cmdk_trimmed_arc(default_value.into()));
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = self.open;
        let open_model = open.clone();
        let query = self.query;
        let query_model = query.clone();
        let open_change_reason_cell = cx.slot_state(
            || Arc::new(std::sync::Mutex::new(None::<CommandDialogOpenChangeReason>)),
            |cell| cell.clone(),
        );
        let pending_dispatch_cell = cx.slot_state(
            || Arc::new(std::sync::Mutex::new(None::<PendingCommandDispatch>)),
            |cell| cell.clone(),
        );
        let close_complete_cell = cx.slot_state(
            || Arc::new(std::sync::Mutex::new(false)),
            |cell| cell.clone(),
        );
        let entries = self.entries;
        let a11y_label = self
            .a11y_label
            .unwrap_or_else(|| Arc::from("Command palette"));
        let disabled = self.disabled;
        let should_filter = self.should_filter;
        let filter = self.filter;
        let value = self.value;
        let default_value = self.default_value;
        let on_value_change = self.on_value_change;
        let wrap = self.wrap;
        let vim_bindings = self.vim_bindings;
        let disable_pointer_selection = self.disable_pointer_selection;
        let close_on_select = self.close_on_select;
        let empty_text = self.empty_text;
        let on_open_change = self.on_open_change;
        let on_open_change_with_reason = self.on_open_change_with_reason;
        let on_open_change_complete = self.on_open_change_complete;

        let on_open_change_with_reason_for_dialog = on_open_change_with_reason.clone();
        let open_change_reason_cell_for_open_change = open_change_reason_cell.clone();
        let on_open_change_for_dialog = on_open_change.clone();
        let on_open_change_complete_for_dialog = on_open_change_complete.clone();
        let close_complete_cell_for_open_change_complete = close_complete_cell.clone();
        let dialog_on_open_change_complete: Option<OnOpenChange> = Some(Arc::new(move |is_open| {
            if let Ok(mut slot) = close_complete_cell_for_open_change_complete.lock() {
                *slot = !is_open;
            }
            if let Some(handler) = on_open_change_complete_for_dialog.as_ref() {
                handler(is_open);
            }
        }));

        let is_open = cx.watch_model(&open_model).layout().copied_or(false);
        let should_dispatch = if is_open {
            false
        } else if let Ok(mut close_complete) = close_complete_cell.lock() {
            if *close_complete {
                *close_complete = false;
                true
            } else {
                false
            }
        } else {
            false
        };

        if should_dispatch {
            if let Ok(mut slot) = pending_dispatch_cell.lock() {
                if let Some(pending) = slot.take() {
                    let kind = match pending.reason {
                        ActivateReason::Pointer => {
                            fret_runtime::CommandDispatchSourceKindV1::Pointer
                        }
                        ActivateReason::Keyboard => {
                            fret_runtime::CommandDispatchSourceKindV1::Keyboard
                        }
                    };
                    cx.app.with_global_mut(
                        fret_runtime::WindowPendingCommandDispatchSourceService::default,
                        |svc, app| {
                            svc.record(
                                cx.window,
                                app.tick_id(),
                                pending.command.clone(),
                                fret_runtime::CommandDispatchSourceV1 {
                                    kind,
                                    element: None,
                                    test_id: None,
                                },
                            );
                        },
                    );
                    cx.app.push_effect(fret_runtime::Effect::Command {
                        window: Some(cx.window),
                        command: pending.command,
                    });
                }
            }
        }

        let dialog_on_open_change: Option<OnOpenChange> = if on_open_change_for_dialog.is_none()
            && on_open_change_with_reason_for_dialog.is_none()
        {
            None
        } else {
            Some(Arc::new(move |is_open| {
                if let Some(handler) = on_open_change_for_dialog.as_ref() {
                    handler(is_open);
                }
                if let Some(handler) = on_open_change_with_reason_for_dialog.as_ref() {
                    let mut reason = CommandDialogOpenChangeReason::None;
                    if let Ok(mut slot) = open_change_reason_cell_for_open_change.lock() {
                        if let Some(stored) = slot.take() {
                            reason = stored;
                        }
                    }
                    if is_open && reason == CommandDialogOpenChangeReason::None {
                        reason = CommandDialogOpenChangeReason::TriggerPress;
                    }
                    handler(is_open, reason);
                }
            }))
        };

        Dialog::new(open)
            .on_open_change(dialog_on_open_change)
            .on_open_change_complete(dialog_on_open_change_complete)
            .on_dismiss_request(Some(Arc::new({
                let open_change_reason_cell = open_change_reason_cell.clone();
                move |_host, _cx, req| {
                    if let Ok(mut slot) = open_change_reason_cell.lock() {
                        *slot = Some(command_dialog_open_change_reason_from_dismiss_reason(
                            req.reason,
                        ));
                    }
                }
            })))
            .into_element(cx, trigger, move |cx| {
                // shadcn/ui v4: command dialog list is `max-h-[300px]` and is allowed to overflow the
                // viewport (the web implementation does not clamp it to the viewport height).
                let list_h = Px(300.0);

                let entries = if close_on_select {
                    let close_action: fret_ui::action::OnActivate = Arc::new({
                        let open_model = open_model.clone();
                        let query_model = query_model.clone();
                        let open_change_reason_cell = open_change_reason_cell.clone();
                        move |host, action_cx, _reason| {
                            if let Ok(mut slot) = open_change_reason_cell.lock() {
                                *slot = Some(CommandDialogOpenChangeReason::ItemPress);
                            }
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
                            CommandEntry::Loading(loading) => CommandEntry::Loading(loading),
                        })
                        .collect()
                } else {
                    entries
                };

                let mut palette = CommandPalette::new(query, Vec::new())
                    .command_dialog_defaults()
                    .entries(entries)
                    .a11y_label(a11y_label.clone())
                    .disabled(disabled)
                    .should_filter(should_filter)
                    .value(value.clone())
                    .on_value_change(on_value_change.clone())
                    .wrap(wrap)
                    .vim_bindings(vim_bindings)
                    .disable_pointer_selection(disable_pointer_selection)
                    .empty_text(empty_text)
                    .refine_scroll_layout(LayoutRefinement::default().h_px(list_h).max_h(list_h));

                if close_on_select {
                    palette = palette.pending_dispatch(pending_dispatch_cell.clone());
                }

                if let Some(default_value) = default_value.as_ref() {
                    palette = palette.default_value(default_value.clone());
                }

                if let Some(filter) = filter.as_ref() {
                    let filter = filter.clone();
                    palette = palette
                        .filter(move |value, search, keywords| filter(value, search, keywords));
                }

                let palette = palette.into_element(cx);

                DialogContent::new(vec![palette])
                    .show_close_button(false)
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .a11y_label(a11y_label)
                    .into_element(cx)
            })
    }
}

#[derive(Default)]
struct CommandPaletteState {
    items_fingerprint: u64,
}

#[derive(Default)]
struct CommandPaletteValueChangeCallbackState {
    initialized: bool,
    last_value: Option<Arc<str>>,
}

fn command_palette_value_change_event(
    state: &mut CommandPaletteValueChangeCallbackState,
    value: Option<Arc<str>>,
) -> Option<Option<Arc<str>>> {
    if !state.initialized {
        state.initialized = true;
        state.last_value = value;
        return None;
    }

    if state.last_value != value {
        state.last_value = value.clone();
        return Some(value);
    }

    None
}

pub fn command<H: UiHost, I, F>(cx: &mut ElementContext<'_, H>, f: F) -> AnyElement
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    Command::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::sync::Mutex;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, SemanticsRole, Size, SvgId,
        SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_ui::tree::UiTree;

    fn snapshot_contains_text(snap: &fret_core::SemanticsSnapshot, text: &str) -> bool {
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::Text
                && (n.label.as_deref() == Some(text) || n.value.as_deref() == Some(text))
        })
    }

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
    fn command_root_defaults_to_fill_width_and_hidden_overflow() {
        use fret_ui::element::ElementKind;
        use fret_ui::elements::GlobalElementId;

        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let command = Command::new(vec![AnyElement::new(
                GlobalElementId(1),
                ElementKind::Text(fret_ui::element::TextProps::new("item")),
                Vec::new(),
            )])
            .into_element(cx);

            let ElementKind::Container(props) = command.kind else {
                panic!("expected Command to build a Container element");
            };

            assert!(matches!(props.layout.size.width, Length::Fill));
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert!(matches!(props.layout.overflow, Overflow::Clip));
        });
    }

    #[test]
    fn command_catalog_item_mapping_preserves_catalog_fields() {
        let command = CommandId::from("app.open");
        let item = CommandItem::from(UiKitCommandCatalogItem {
            label: Arc::from("Open"),
            value: Arc::from("app.open"),
            disabled: true,
            keywords: vec![Arc::from("file"), Arc::from("open")],
            shortcut: Some(Arc::from("Cmd+O")),
            command: command.clone(),
        });

        assert_eq!(item.label.as_ref(), "Open");
        assert_eq!(item.value.as_ref(), "app.open");
        assert!(item.disabled);
        assert_eq!(item.keywords.len(), 2);
        assert_eq!(item.shortcut.as_deref(), Some("Cmd+O"));
        assert_eq!(item.command.as_ref(), Some(&command));
    }

    #[test]
    fn command_catalog_group_mapping_preserves_heading_and_items() {
        let entry = CommandEntry::from(UiKitCommandCatalogEntry::Group(
            UiKitCommandCatalogGroup::new(
                "File",
                vec![UiKitCommandCatalogItem {
                    label: Arc::from("Open"),
                    value: Arc::from("app.open"),
                    disabled: false,
                    keywords: vec![Arc::from("open")],
                    shortcut: None,
                    command: CommandId::from("app.open"),
                }],
            ),
        ));

        match entry {
            CommandEntry::Group(group) => {
                assert_eq!(group.heading.as_deref(), Some("File"));
                assert_eq!(group.items.len(), 1);
                assert_eq!(group.items[0].label.as_ref(), "Open");
            }
            _ => panic!("expected group entry"),
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
    fn command_dialog_open_change_builders_set_handlers() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let query = app.models_mut().insert(String::new());

        let dialog = CommandDialog::new(open, query, Vec::new())
            .on_open_change(Some(Arc::new(|_open| {})))
            .on_open_change_with_reason(Some(Arc::new(|_open, _reason| {})))
            .on_open_change_complete(Some(Arc::new(|_open| {})));

        assert!(dialog.on_open_change.is_some());
        assert!(dialog.on_open_change_with_reason.is_some());
        assert!(dialog.on_open_change_complete.is_some());
    }

    #[test]
    fn command_dialog_open_change_reason_maps_dismiss_reasons() {
        use fret_ui::action::DismissReason;

        assert_eq!(
            command_dialog_open_change_reason_from_dismiss_reason(DismissReason::Escape),
            CommandDialogOpenChangeReason::EscapeKey
        );
        assert_eq!(
            command_dialog_open_change_reason_from_dismiss_reason(DismissReason::OutsidePress {
                pointer: None,
            }),
            CommandDialogOpenChangeReason::OutsidePress
        );
        assert_eq!(
            command_dialog_open_change_reason_from_dismiss_reason(DismissReason::FocusOutside),
            CommandDialogOpenChangeReason::FocusOut
        );
        assert_eq!(
            command_dialog_open_change_reason_from_dismiss_reason(DismissReason::Scroll),
            CommandDialogOpenChangeReason::None
        );
    }

    #[test]
    fn command_dialog_open_change_with_reason_reports_item_press_when_close_on_select() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::from("al"));
        let reasons: Arc<std::sync::Mutex<Vec<(bool, CommandDialogOpenChangeReason)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        let bounds = bounds();
        let mut services = FakeServices::default();

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);
            crate::shadcn_themes::apply_shadcn_new_york(
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
                "cmdk-dialog-reason",
                |cx| {
                    vec![
                        CommandDialog::new(
                            open.clone(),
                            query.clone(),
                            vec![CommandItem::new("Alpha")],
                        )
                        .close_on_select(true)
                        .on_open_change_with_reason(Some(Arc::new({
                            let reasons = reasons.clone();
                            move |is_open, reason| {
                                reasons
                                    .lock()
                                    .expect("reasons lock")
                                    .push((is_open, reason));
                            }
                        })))
                        .into_element(cx, |cx| crate::Button::new("Open").into_element(cx)),
                    ]
                },
            );
            ui.set_root(root);
            fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Alpha"))
            .map(|n| n.bounds)
            .expect("Alpha option bounds");
        click(&mut ui, &mut app, &mut services, rect_center(alpha));

        render_frame(&mut ui, &mut app, &mut services);

        let captured = reasons.lock().expect("reasons lock").clone();
        assert!(
            captured.iter().any(|(is_open, reason)| !*is_open
                && *reason == CommandDialogOpenChangeReason::ItemPress)
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

        crate::shadcn_themes::apply_shadcn_new_york(
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
    fn command_item_on_select_value_action_receives_value() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::new());
        let selected_value = app.models_mut().insert(None::<Arc<str>>);

        let build_items = || {
            vec![
                CommandItem::new("Alpha")
                    .value("alpha-id")
                    .on_select_value_action({
                        let selected_value = selected_value.clone();
                        move |host, action_cx, _reason, value| {
                            let _ = host.models_mut().update(&selected_value, |cur| {
                                *cur = Some(value.clone());
                            });
                            host.request_redraw(action_cx.window);
                        }
                    }),
            ]
        };

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
            build_items(),
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

        let _root = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            build_items(),
            false,
        );

        assert_eq!(
            selected_value
                .read_ref(&app, |v| v.clone())
                .expect("read selected value")
                .as_deref(),
            Some("alpha-id")
        );
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

        let build_items = || vec![CommandItem::new("Alpha").on_select_action(on_select.clone())];

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
            build_items(),
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
            build_items(),
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

        let build_items = || vec![CommandItem::new("Alpha").on_select_action(on_select.clone())];

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
            build_items(),
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
            build_items(),
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

        let build_items = || {
            (0..80)
                .map(|i| CommandItem::new(format!("Item {i}")))
                .collect::<Vec<_>>()
        };

        // First frame: mount overlay/content.
        let _ = render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            query.clone(),
            build_items(),
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
            build_items(),
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
            build_items(),
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

    #[test]
    fn command_dialog_content_stays_within_panel_bounds_on_sm_viewport() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        let build_items = || {
            vec![
                CommandItem::new(
                    "Long command label that should still measure inside the command dialog panel",
                ),
                CommandItem::new("Secondary action"),
            ]
        };

        for _ in 1..=3 {
            let _ = render_dialog_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                query.clone(),
                build_items(),
                true,
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let dialog = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Dialog)
            .expect("dialog node");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox)
            .expect("combobox input node");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        let option = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::ListBoxOption
                    && n.label.as_deref()
                        == Some(
                            "Long command label that should still measure inside the command dialog panel",
                        )
            })
            .expect("long option node");

        let dialog_left = dialog.bounds.origin.x.0 - 0.5;
        let dialog_right = dialog.bounds.origin.x.0 + dialog.bounds.size.width.0 + 0.5;

        assert!(
            dialog.bounds.size.width.0 <= 512.5,
            "expected command dialog width to stay near shadcn's sm:max-w-lg, got {:?}",
            dialog.bounds
        );
        assert!(
            input.bounds.origin.x.0 >= dialog_left
                && input.bounds.origin.x.0 + input.bounds.size.width.0 <= dialog_right,
            "expected command dialog input to stay inside dialog panel; dialog={:?} input={:?}",
            dialog.bounds,
            input.bounds
        );
        assert!(
            list.bounds.origin.x.0 >= dialog_left
                && list.bounds.origin.x.0 + list.bounds.size.width.0 <= dialog_right,
            "expected command dialog list to stay inside dialog panel; dialog={:?} list={:?}",
            dialog.bounds,
            list.bounds
        );
        assert!(
            option.bounds.origin.x.0 >= dialog_left
                && option.bounds.origin.x.0 + option.bounds.size.width.0 <= dialog_right,
            "expected command dialog option to stay inside dialog panel; dialog={:?} option={:?}",
            dialog.bounds,
            option.bounds
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

    fn render_frame_with_value(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<String>,
        value: Model<Option<Arc<str>>>,
        items: Vec<CommandItem>,
    ) -> fret_core::NodeId {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "cmdk", |cx| {
                vec![
                    CommandPalette::new(model, items)
                        .value(Some(value.clone()))
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_value_and_on_value_change(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<String>,
        value: Model<Option<Arc<str>>>,
        on_value_change: Option<OnValueChange>,
        items: Vec<CommandItem>,
    ) -> fret_core::NodeId {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "cmdk", |cx| {
                vec![
                    CommandPalette::new(model, items)
                        .value(Some(value.clone()))
                        .on_value_change(on_value_change.clone())
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_disable_pointer_selection(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<String>,
        items: Vec<CommandItem>,
        disable_pointer_selection: bool,
    ) -> fret_core::NodeId {
        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "cmdk", |cx| {
                vec![
                    CommandPalette::new(model, items)
                        .disable_pointer_selection(disable_pointer_selection)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn row_signatures(
        rows: &[CommandPaletteRenderRow],
        items: &[Option<CommandItem>],
    ) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                CommandPaletteRenderRow::Heading(h) => format!("H:{h}"),
                CommandPaletteRenderRow::GroupPad => "P".to_string(),
                CommandPaletteRenderRow::Separator(_) => "S".to_string(),
                CommandPaletteRenderRow::Loading(_) => "L".to_string(),
                CommandPaletteRenderRow::Item(idx) => {
                    let label = items
                        .get(*idx)
                        .and_then(|i| i.as_ref())
                        .map(|i| i.label.as_ref())
                        .unwrap_or("<missing>");
                    format!("I:{label}")
                }
            })
            .collect()
    }

    #[test]
    fn command_palette_loading_row_is_preserved_in_render_rows() {
        let entries = vec![
            CommandLoading::new("Hang on…").into(),
            CommandItem::new("Alpha").into(),
            CommandSeparator::new().into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "", true, None);

        assert_eq!(items.len(), 1);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["L".to_string(), "I:Alpha".to_string()]
        );
    }

    #[test]
    fn cmdk_loading_row_renders_when_no_items() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let query = app.models_mut().insert(String::new());
        let mut services = FakeServices::default();
        let b = bounds();

        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "cmdk",
            |cx| {
                vec![
                    CommandPalette::new(query.clone(), Vec::<CommandItem>::new())
                        .entries(vec![
                            CommandLoading::new("Fetching…")
                                .progress(50)
                                .test_id("cmdk-loading")
                                .into(),
                        ])
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let loading = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("cmdk-loading"))
            .expect("loading row node");
        assert_eq!(loading.role, SemanticsRole::ProgressBar);
        assert_eq!(loading.label.as_deref(), Some("Fetching…"));
        assert_eq!(loading.value.as_deref(), Some("50%"));
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

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
            ]
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
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
            build_items(),
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
    fn command_palette_test_id_prefix_derives_surface_ids() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let query = app.models_mut().insert(String::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "cmdk-test-id-prefix",
            |cx| {
                vec![
                    CommandPalette::new(
                        query.clone(),
                        vec![CommandItem::new("Alpha"), CommandItem::new("Beta")],
                    )
                    .test_id_prefix("cmd")
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let ids: Vec<&str> = snap
            .nodes
            .iter()
            .filter_map(|n| n.test_id.as_deref())
            .collect();
        assert!(ids.iter().copied().any(|id| id == "cmd-input"));
        assert!(ids.iter().copied().any(|id| id == "cmd-listbox"));
        assert!(ids.iter().copied().any(|id| id == "cmd-item-alpha"));

        let _ = root;
    }

    #[test]
    fn cmdk_renders_empty_state_when_filtered_to_no_items() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let query = app.models_mut().insert(String::from("zzz"));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query,
            vec![CommandItem::new("Alpha"), CommandItem::new("Beta")],
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::ListBox),
            "expected cmdk empty state to keep listbox semantics (cmdk-compatible structure)"
        );
        assert!(
            snap.nodes
                .iter()
                .all(|n| n.role != SemanticsRole::ListBoxOption),
            "expected cmdk empty state to render without listbox options"
        );
        assert!(
            snapshot_contains_text(&snap, "No results."),
            "expected cmdk empty state to render the default empty text"
        );
    }

    #[test]
    fn cmdk_on_value_change_emits_active_value_changes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(String::new());
        let value = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let seen: Arc<Mutex<Vec<Option<Arc<str>>>>> = Arc::new(Mutex::new(Vec::new()));
        let on_value_change: OnValueChange = Arc::new({
            let seen = seen.clone();
            move |value| {
                seen.lock().unwrap_or_else(|e| e.into_inner()).push(value);
            }
        });
        let on_value_change_opt = Some(on_value_change.clone());

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
                CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
            ]
        };

        let root = render_frame_with_value_and_on_value_change(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            value.clone(),
            on_value_change_opt.clone(),
            build_items(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        assert!(
            seen.lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
            "expected initial mount to not emit on_value_change"
        );

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

        let _ = render_frame_with_value_and_on_value_change(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            value.clone(),
            on_value_change_opt,
            build_items(),
        );

        let seen = seen.lock().unwrap_or_else(|e| e.into_inner()).clone();
        assert_eq!(
            seen.last().and_then(|v| v.as_deref()),
            Some("Beta"),
            "expected ArrowDown to emit the active cmdk value"
        );
        assert_eq!(
            app.models().get_cloned(&value).flatten().as_deref(),
            Some("Beta"),
            "expected ArrowDown to update the active value model"
        );
    }

    #[test]
    fn cmdk_value_model_respects_controlled_selection() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let query = app.models_mut().insert(String::new());
        let selected = app.models_mut().insert(Some(Arc::<str>::from("Beta")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
            ]
        };

        let root = render_frame_with_value(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query.clone(),
            selected.clone(),
            build_items(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        let _ = render_frame_with_value(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query,
            selected.clone(),
            build_items(),
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

        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(
            app.models()
                .get_cloned(&selected)
                .unwrap_or(None)
                .as_deref(),
            Some("Beta")
        );
    }

    #[test]
    fn cmdk_value_model_updates_on_arrow_navigation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let query = app.models_mut().insert(String::new());
        let selected = app.models_mut().insert(Some(Arc::<str>::from("Alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
            ]
        };

        let root = render_frame_with_value(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query.clone(),
            selected.clone(),
            build_items(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame_with_value(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query,
            selected.clone(),
            build_items(),
        );

        assert_eq!(
            app.models()
                .get_cloned(&selected)
                .unwrap_or(None)
                .as_deref(),
            Some("Beta")
        );
    }

    #[test]
    fn cmdk_force_mount_item_survives_filtering() {
        let entries = vec![
            CommandItem::new("Calendar")
                .keywords(["events"])
                .on_select("calendar")
                .into(),
            CommandItem::new("Force mounted row")
                .value("force-mounted")
                .force_mount(true)
                .on_select("force-mounted")
                .into(),
        ];

        let (_rows, items, _groups) =
            command_palette_render_rows_for_query_with_options(entries, "zzz", true, None);

        assert!(items.iter().any(|item| {
            item.as_ref()
                .is_some_and(|item| item.value.as_ref() == "force-mounted")
        }));
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
                                LayoutRefinement::default().h_px(Px(40.0)).max_h(Px(40.0)),
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

        let build_items = || {
            (0..12)
                .map(|i| {
                    CommandItem::new(format!("Item {i}")).on_select(CommandId::new(format!("i{i}")))
                })
                .collect::<Vec<_>>()
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
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
            build_items(),
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

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
                CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
            ]
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta row bounds");
        let beta_id = beta.id;
        let beta_bounds = beta.bounds;

        let debug = std::env::var("FRET_DEBUG_CMDK_HOVER").is_ok();
        if debug {
            let pos = Point::new(
                Px(beta_bounds.origin.x.0 + 1.0),
                Px(beta_bounds.origin.y.0 + 1.0),
            );
            eprintln!("cmdk hover debug: beta_bounds={beta_bounds:?} pos={pos:?}");
            let window_id = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Window)
                .map(|n| n.id);
            let window_bounds = window_id.and_then(|id| ui.debug_node_bounds(id));
            let beta_ui_bounds = ui.debug_node_bounds(beta_id);
            eprintln!("cmdk hover debug: window_id={window_id:?} window_bounds={window_bounds:?}");
            eprintln!("cmdk hover debug: beta_id={beta_id:?} beta_ui_bounds={beta_ui_bounds:?}");
            let input_ui_bounds = ui.debug_node_bounds(input);
            if let Some(input_ui_bounds) = input_ui_bounds {
                let input_pos = Point::new(
                    Px(input_ui_bounds.origin.x.0 + 1.0),
                    Px(input_ui_bounds.origin.y.0 + 1.0),
                );
                eprintln!(
                    "cmdk hover debug: input={input:?} input_ui_bounds={input_ui_bounds:?} input_hit_test={:?}",
                    ui.debug_hit_test(input_pos)
                );
                let mut input_chain: Vec<NodeId> = Vec::new();
                let mut current = Some(input);
                for _ in 0..32 {
                    let Some(id) = current else {
                        break;
                    };
                    input_chain.push(id);
                    current = ui.node_parent(id);
                }
                eprintln!("cmdk hover debug: input parent chain={input_chain:?}");
            } else {
                eprintln!("cmdk hover debug: input={input:?} input_ui_bounds=None");
            }
            let mut chain: Vec<NodeId> = Vec::new();
            let mut current = Some(beta_id);
            for _ in 0..32 {
                let Some(id) = current else {
                    break;
                };
                chain.push(id);
                current = ui.node_parent(id);
            }
            eprintln!("cmdk hover debug: beta parent chain={chain:?}");
            for id in &chain {
                let bounds = ui.debug_node_bounds(*id);
                let contains = bounds.is_some_and(|b| b.contains(pos));
                let kind = ui.debug_declarative_instance_kind(&mut app, window, *id);
                let measured = ui.debug_node_measured_size(*id);
                let role = snap.nodes.iter().find(|n| n.id == *id).map(|n| n.role);
                let label = snap
                    .nodes
                    .iter()
                    .find(|n| n.id == *id)
                    .and_then(|n| n.label.clone());
                eprintln!(
                    "cmdk hover debug: chain node={id:?} kind={kind:?} measured={measured:?} role={role:?} label={label:?} bounds={bounds:?} contains_pos={contains}"
                );
            }
            eprintln!(
                "cmdk hover debug: layers={:?}",
                ui.debug_layers_in_paint_order()
            );
            eprintln!("cmdk hover debug: hit_test={:?}", ui.debug_hit_test(pos));
        }

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
            build_items(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        if debug {
            let pos = Point::new(
                Px(beta_bounds.origin.x.0 + 1.0),
                Px(beta_bounds.origin.y.0 + 1.0),
            );
            let hit = ui.debug_hit_test(pos).hit;
            let hit_label = hit.and_then(|id| {
                snap.nodes
                    .iter()
                    .find(|n| n.id == id)
                    .and_then(|n| n.label.clone())
            });
            let hit_role =
                hit.and_then(|id| snap.nodes.iter().find(|n| n.id == id).map(|n| n.role));
            eprintln!(
                "cmdk hover debug: post-render hit={hit:?} role={hit_role:?} label={hit_label:?}"
            );
        }

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
    fn cmdk_disable_pointer_selection_prevents_hover_highlight() {
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

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
                CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
            ]
        };

        let root = render_frame_disable_pointer_selection(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
            true,
        );

        let input = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable text input");
        ui.set_focus(Some(input));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta row bounds");
        let beta_bounds = beta.bounds;

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

        let _ = render_frame_disable_pointer_selection(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
            true,
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
        assert_eq!(active_node.label.as_deref(), Some("Alpha"));
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

        let build_items = || {
            vec![
                CommandItem::new("Alpha").on_select(CommandId::new("alpha")),
                CommandItem::new("Beta").on_select(CommandId::new("beta")),
                CommandItem::new("Gamma").on_select(CommandId::new("gamma")),
            ]
        };

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            build_items(),
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
            build_items(),
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

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "", true, None);
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

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "gam", true, None);
        assert_eq!(items.len(), 1);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["H:Advanced".to_string(), "I:Gamma".to_string()]
        );
    }

    #[test]
    fn cmdk_group_force_mount_keeps_group_visible_when_filtered() {
        let entries = vec![
            CommandGroup::new(vec![CommandItem::new("Alpha"), CommandItem::new("Beta")])
                .heading("Letters")
                .force_mount(true)
                .into(),
            CommandGroup::new(vec![CommandItem::new("Giraffe")])
                .heading("Animals")
                .into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "gir", true, None);
        let sigs = row_signatures(&rows, &items);
        assert!(sigs.iter().any(|sig| sig == "H:Letters"));
        assert!(sigs.iter().any(|sig| sig == "I:Giraffe"));
        assert!(!sigs.iter().any(|sig| sig == "I:Alpha"));
    }

    #[test]
    fn command_palette_sort_orders_ungrouped_items_by_score() {
        let entries = vec![
            CommandItem::new("pal").into(),
            CommandItem::new("alpha").into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "al", true, None);
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

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "al", true, None);
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

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "al", true, None);
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

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "", true, None);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["I:Alpha".to_string(), "S".to_string(), "I:Beta".to_string()]
        );
    }

    #[test]
    fn cmdk_separator_always_render_stays_visible_with_search() {
        let entries = vec![
            CommandItem::new("Alpha").into(),
            CommandSeparator::new().always_render(true).into(),
            CommandItem::new("Beta").into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "al", true, None);
        assert!(row_signatures(&rows, &items).iter().any(|sig| sig == "S"));
    }

    #[test]
    fn cmdk_should_filter_false_shows_all_items_even_when_query_non_empty() {
        let entries = vec![
            CommandItem::new("Alpha").into(),
            CommandSeparator::new().into(),
            CommandItem::new("Beta").into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "zzz", false, None);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["I:Alpha".to_string(), "I:Beta".to_string()]
        );
    }

    #[test]
    fn cmdk_should_filter_false_hides_separators_unless_always_render() {
        let entries = vec![
            CommandItem::new("Alpha").into(),
            CommandSeparator::new().always_render(true).into(),
            CommandItem::new("Beta").into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "zzz", false, None);
        assert!(row_signatures(&rows, &items).iter().any(|sig| sig == "S"));
    }

    #[test]
    fn cmdk_should_filter_false_preserves_entry_order_instead_of_sorting() {
        let entries = vec![
            CommandItem::new("pal").into(),
            CommandItem::new("alpha").into(),
        ];

        let (rows, items, _item_groups) =
            command_palette_render_rows_for_query_with_options(entries, "al", false, None);
        assert_eq!(
            row_signatures(&rows, &items),
            vec!["I:pal".to_string(), "I:alpha".to_string()]
        );
    }

    #[test]
    fn cmdk_filter_callback_controls_visibility_and_score() {
        let entries = vec![
            CommandItem::new("Alpha").into(),
            CommandItem::new("Beta").into(),
        ];
        let filter: CommandPaletteFilterFn =
            Arc::new(|value, _search, _keywords| if value == "Beta" { 1.0 } else { 0.0 });

        let (rows, items, _item_groups) = command_palette_render_rows_for_query_with_options(
            entries,
            "x",
            true,
            Some(filter.as_ref()),
        );
        assert_eq!(row_signatures(&rows, &items), vec!["I:Beta".to_string()]);
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
