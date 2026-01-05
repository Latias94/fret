use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, SemanticsProps, TextProps,
};
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome as decl_chrome;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::{roving_focus, typeahead};
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::recipes::input::{
    InputTokenKeys, input_chrome_container_props, resolve_input_chrome,
};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Space,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

impl From<SelectAlign> for Align {
    fn from(value: SelectAlign) -> Self {
        match value {
            SelectAlign::Start => Align::Start,
            SelectAlign::Center => Align::Center,
            SelectAlign::End => Align::End,
        }
    }
}

impl From<SelectSide> for Side {
    fn from(value: SelectSide) -> Self {
        match value {
            SelectSide::Top => Side::Top,
            SelectSide::Right => Side::Right,
            SelectSide::Bottom => Side::Bottom,
            SelectSide::Left => Side::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// shadcn/ui `SelectLabel` (v4).
#[derive(Debug, Clone)]
pub struct SelectLabel {
    pub text: Arc<str>,
}

impl SelectLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

/// shadcn/ui `SelectGroup` (v4).
///
/// In the upstream DOM implementation this is a structural wrapper. In Fret we render it by
/// flattening its entries into the surrounding listbox.
#[derive(Debug, Clone)]
pub struct SelectGroup {
    pub entries: Vec<SelectEntry>,
}

impl SelectGroup {
    pub fn new(entries: Vec<SelectEntry>) -> Self {
        Self { entries }
    }
}

/// shadcn/ui `SelectSeparator` (v4).
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectSeparator;

#[derive(Debug, Clone)]
pub enum SelectEntry {
    Item(SelectItem),
    Label(SelectLabel),
    Group(SelectGroup),
    Separator(SelectSeparator),
}

impl From<SelectItem> for SelectEntry {
    fn from(value: SelectItem) -> Self {
        Self::Item(value)
    }
}

impl From<SelectLabel> for SelectEntry {
    fn from(value: SelectLabel) -> Self {
        Self::Label(value)
    }
}

impl From<SelectGroup> for SelectEntry {
    fn from(value: SelectGroup) -> Self {
        Self::Group(value)
    }
}

impl From<SelectSeparator> for SelectEntry {
    fn from(value: SelectSeparator) -> Self {
        Self::Separator(value)
    }
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    entries: Vec<SelectEntry>,
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    align: SelectAlign,
    side: SelectSide,
    align_offset: Px,
    side_offset_override: Option<Px>,
    loop_navigation: bool,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            entries: Vec::new(),
            placeholder: Arc::from("Select..."),
            disabled: false,
            a11y_label: None,
            layout: LayoutRefinement::default(),
            align: SelectAlign::default(),
            side: SelectSide::default(),
            align_offset: Px(0.0),
            side_offset_override: None,
            loop_navigation: true,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
        }
    }

    pub fn item(mut self, item: SelectItem) -> Self {
        self.entries.push(SelectEntry::Item(item));
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectItem>) -> Self {
        self.entries
            .extend(items.into_iter().map(SelectEntry::Item));
        self
    }

    pub fn entry(mut self, entry: impl Into<SelectEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = SelectEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn align(mut self, align: SelectAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: SelectSide) -> Self {
        self.side = side;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset_override = Some(offset);
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Enables a Select arrow (Radix `SelectArrow`-style).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        select_impl(
            cx,
            self.model,
            self.open,
            &self.entries,
            self.placeholder,
            self.disabled,
            self.a11y_label,
            self.layout,
            self.align,
            self.side,
            self.align_offset,
            self.side_offset_override,
            self.loop_navigation,
            self.arrow,
            self.arrow_size_override,
            self.arrow_padding_override,
        )
    }
}

pub fn select<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: &[SelectItem],
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
) -> AnyElement {
    let entries: Vec<SelectEntry> = items.iter().cloned().map(SelectEntry::Item).collect();
    select_impl(
        cx,
        model,
        open,
        &entries,
        placeholder,
        disabled,
        a11y_label,
        layout,
        SelectAlign::default(),
        SelectSide::default(),
        Px(0.0),
        None,
        true,
        false,
        None,
        None,
    )
}

fn select_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    entries: &[SelectEntry],
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    align: SelectAlign,
    side: SelectSide,
    align_offset: Px,
    side_offset_override: Option<Px>,
    loop_navigation: bool,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
) -> AnyElement {
    cx.scope(|cx| {
        fn find_item_label(entries: &[SelectEntry], value: &str) -> Option<Arc<str>> {
            for entry in entries {
                match entry {
                    SelectEntry::Item(it) => {
                        if it.value.as_ref() == value {
                            return Some(it.label.clone());
                        }
                    }
                    SelectEntry::Group(group) => {
                        if let Some(label) = find_item_label(&group.entries, value) {
                            return Some(label);
                        }
                    }
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
            None
        }

        fn count_items(entries: &[SelectEntry]) -> usize {
            let mut count: usize = 0;
            for entry in entries {
                match entry {
                    SelectEntry::Item(_) => count = count.saturating_add(1),
                    SelectEntry::Group(group) => count = count.saturating_add(count_items(&group.entries)),
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
            count
        }

        let theme = Theme::global(&*cx.app).clone();
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        let is_open = cx.watch_model(&open).copied().unwrap_or(false);
        let arrow_size = arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.select.arrow_size")
                .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                .unwrap_or(Px(12.0))
        });
        let arrow_padding = arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.select.arrow_padding")
                .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                .unwrap_or(theme.metrics.radius_md)
        });

        let resolved = resolve_input_chrome(
            &theme,
            fret_ui_kit::Size::default(),
            &ChromeRefinement::default(),
            InputTokenKeys::none(),
        );

        let radius = resolved.radius;
        let ring = decl_style::focus_ring(&theme, radius);

        let label = selected
            .as_ref()
            .and_then(|v| find_item_label(entries, v.as_ref()))
            .unwrap_or(placeholder);

        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            weight: FontWeight::NORMAL,
            line_height: theme.metric_by_key("font.line_height").or(Some(theme.metrics.font_line_height)),
            letter_spacing_em: None,
        };

        let min_width = theme
            .metric_by_key("component.select.min_width")
            .unwrap_or(Px(180.0));

        let mut trigger_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .min_w(MetricRef::Px(min_width))
                .min_h(MetricRef::Px(resolved.min_height))
                .merge(layout),
        );
        trigger_layout.size.height = Length::Auto;
        trigger_layout.size.min_height = Some(resolved.min_height);

        let border = resolved.border_color;
        let border_focus = resolved.border_color_focused;
        let fg = resolved.text_color;
        let fg_muted = theme
            .color_by_key("muted-foreground")
            .unwrap_or(theme.colors.text_muted);

        let enabled = !disabled;
        let item_len = count_items(entries);

        #[derive(Debug)]
        struct SelectTriggerKeyState {
            suppress_next_activate: bool,
            query: String,
            clear_token: Option<TimerToken>,
        }

        impl SelectTriggerKeyState {
            fn new() -> Self {
                Self {
                    suppress_next_activate: false,
                    query: String::new(),
                    clear_token: None,
                }
            }
        }

        fn flatten_items_for_typeahead(
            entries: &[SelectEntry],
            enabled: bool,
            values: &mut Vec<Arc<str>>,
            labels: &mut Vec<Arc<str>>,
            disabled: &mut Vec<bool>,
        ) {
            for entry in entries {
                match entry {
                    SelectEntry::Item(item) => {
                        values.push(item.value.clone());
                        labels.push(item.label.clone());
                        disabled.push(item.disabled || !enabled);
                    }
                    SelectEntry::Group(group) => {
                        flatten_items_for_typeahead(&group.entries, enabled, values, labels, disabled);
                    }
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
        }

        decl_chrome::control_chrome_pressable_with_id_props(cx, |cx, st, trigger_id| {
            let mut typeahead_values: Vec<Arc<str>> = Vec::new();
            let mut typeahead_labels: Vec<Arc<str>> = Vec::new();
            let mut typeahead_disabled: Vec<bool> = Vec::new();
            flatten_items_for_typeahead(
                entries,
                enabled,
                &mut typeahead_values,
                &mut typeahead_labels,
                &mut typeahead_disabled,
            );

            let typeahead_values: Arc<[Arc<str>]> = Arc::from(typeahead_values.into_boxed_slice());
            let typeahead_labels: Arc<[Arc<str>]> = Arc::from(typeahead_labels.into_boxed_slice());
            let typeahead_disabled: Arc<[bool]> = Arc::from(typeahead_disabled.into_boxed_slice());

            let trigger_state: Arc<Mutex<SelectTriggerKeyState>> = cx.with_state_for(
                trigger_id,
                || Arc::new(Mutex::new(SelectTriggerKeyState::new())),
                |s| s.clone(),
            );

            let state_for_timer = trigger_state.clone();
            cx.timer_on_timer_for(
                trigger_id,
                Arc::new(move |_host, _action_cx, token| {
                    let mut state = state_for_timer
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if state.clear_token == Some(token) {
                        state.clear_token = None;
                        state.query.clear();
                        return true;
                    }
                    false
                }),
            );

            let open_for_key = open.clone();
            let model_for_key = model.clone();
            let values_for_key = typeahead_values.clone();
            let labels_for_key = typeahead_labels.clone();
            let disabled_for_key = typeahead_disabled.clone();
            let state_for_key = trigger_state.clone();
            cx.key_on_key_down_for(
                trigger_id,
                Arc::new(move |host, action_cx, it| {
                    use fret_core::KeyCode;

                    if it.repeat {
                        return false;
                    }

                    let is_open = host.models_mut().get_copied(&open_for_key).unwrap_or(false);
                    if is_open {
                        return false;
                    }

                    let is_modifier_key = it.modifiers.ctrl
                        || it.modifiers.alt
                        || it.modifiers.meta
                        || it.modifiers.alt_gr;
                    if is_modifier_key {
                        return false;
                    }

                    let mut state = state_for_key
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());

                    if it.key == KeyCode::Space && !state.query.is_empty() {
                        return true;
                    }

                    if matches!(
                        it.key,
                        KeyCode::Enter | KeyCode::Space | KeyCode::ArrowDown | KeyCode::ArrowUp
                    ) {
                        if matches!(it.key, KeyCode::Enter | KeyCode::Space) {
                            state.suppress_next_activate = true;
                        }
                        if let Some(token) = state.clear_token.take() {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        state.query.clear();

                        let _ = host.models_mut().update(&open_for_key, |v| *v = true);
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    let key_to_ascii = |key: fret_core::KeyCode| -> Option<char> {
                        use fret_core::KeyCode;
                        Some(match key {
                            KeyCode::KeyA => 'a',
                            KeyCode::KeyB => 'b',
                            KeyCode::KeyC => 'c',
                            KeyCode::KeyD => 'd',
                            KeyCode::KeyE => 'e',
                            KeyCode::KeyF => 'f',
                            KeyCode::KeyG => 'g',
                            KeyCode::KeyH => 'h',
                            KeyCode::KeyI => 'i',
                            KeyCode::KeyJ => 'j',
                            KeyCode::KeyK => 'k',
                            KeyCode::KeyL => 'l',
                            KeyCode::KeyM => 'm',
                            KeyCode::KeyN => 'n',
                            KeyCode::KeyO => 'o',
                            KeyCode::KeyP => 'p',
                            KeyCode::KeyQ => 'q',
                            KeyCode::KeyR => 'r',
                            KeyCode::KeyS => 's',
                            KeyCode::KeyT => 't',
                            KeyCode::KeyU => 'u',
                            KeyCode::KeyV => 'v',
                            KeyCode::KeyW => 'w',
                            KeyCode::KeyX => 'x',
                            KeyCode::KeyY => 'y',
                            KeyCode::KeyZ => 'z',
                            KeyCode::Digit0 => '0',
                            KeyCode::Digit1 => '1',
                            KeyCode::Digit2 => '2',
                            KeyCode::Digit3 => '3',
                            KeyCode::Digit4 => '4',
                            KeyCode::Digit5 => '5',
                            KeyCode::Digit6 => '6',
                            KeyCode::Digit7 => '7',
                            KeyCode::Digit8 => '8',
                            KeyCode::Digit9 => '9',
                            _ => return None,
                        })
                    };

                    let Some(ch) = key_to_ascii(it.key) else {
                        return false;
                    };

                    state.query.push(ch);
                    if let Some(token) = state.clear_token.take() {
                        host.push_effect(Effect::CancelTimer { token });
                    }
                    let token = host.next_timer_token();
                    state.clear_token = Some(token);
                    host.push_effect(Effect::SetTimer {
                        window: Some(action_cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });

                    let current = host
                        .models_mut()
                        .read(&model_for_key, |v| v.clone())
                        .ok()
                        .flatten();
                    let current_idx = current.as_ref().and_then(|v| {
                        values_for_key
                            .iter()
                            .position(|it| it.as_ref() == v.as_ref())
                    });

                    if let Some(next) = typeahead::match_prefix_arc_str(
                        labels_for_key.as_ref(),
                        disabled_for_key.as_ref(),
                        &state.query,
                        current_idx,
                        true,
                    ) && let Some(next_value) = values_for_key.get(next).cloned()
                    {
                        let _ = host
                            .models_mut()
                            .update(&model_for_key, |v| *v = Some(next_value));
                        host.request_redraw(action_cx.window);
                    }

                    true
                }),
            );

            let open_for_activate = open.clone();
            let state_for_activate = trigger_state.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut state = state_for_activate
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());

                if state.suppress_next_activate {
                    state.suppress_next_activate = false;
                    return;
                }

                if let Some(token) = state.clear_token.take() {
                    host.push_effect(Effect::CancelTimer { token });
                }
                state.query.clear();

                let _ = host.models_mut().update(&open_for_activate, |v| *v = !*v);
                host.request_redraw(action_cx.window);
            }));

            let border_color = if st.hovered || st.pressed || st.focused {
                alpha_mul(border_focus, 0.85)
            } else {
                border
            };

            let props = PressableProps {
                layout: trigger_layout,
                enabled,
                focusable: enabled,
                focus_ring: Some(ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ComboBox),
                    label: a11y_label.clone(),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            };

            let overlay_root_name = OverlayController::popover_root_name(trigger_id);

            if is_open
                && enabled
                && let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id)
            {
                    let window_margin = theme
                        .metric_by_key("component.select.window_margin")
                        .unwrap_or(Px(8.0));
                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let max_h = theme
                        .metric_by_key("component.select.max_list_height")
                        .unwrap_or(Px(240.0));
                    let item_h = theme
                        .metric_by_key("component.select.item_height")
                        .unwrap_or(Px(32.0));
                    let desired_h =
                        Px((item_h.0 * item_len as f32).min(max_h.0).max(item_h.0));
                    let desired_w = Px(anchor.size.width.0.max(min_width.0));
                    let desired = fret_core::Size::new(desired_w, desired_h);

                    let side_offset = side_offset_override.unwrap_or_else(|| {
                        theme
                            .metric_by_key("component.select.popover_offset")
                            .unwrap_or(Px(6.0))
                    });

                    let border_width = resolved.border_width;
                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let layout = popper::popper_content_layout_sized(
                        outer,
                        anchor,
                        desired,
                        popper::PopperContentPlacement::new(
                            LayoutDirection::Ltr,
                            side.into(),
                            align.into(),
                            side_offset,
                        )
                        .with_align_offset(align_offset)
                        .with_arrow(arrow_options, arrow_protrusion),
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);

                    let theme_for_overlay = theme.clone();
                    let text_style_for_overlay = text_style.clone();
                    let open_for_overlay = open.clone();

                    let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                        let selected = cx.watch_model(&model).cloned().unwrap_or_default();

                        #[derive(Clone)]
                        enum SelectRow {
                            Item(SelectItem),
                            Label(SelectLabel),
                            Separator,
                        }

                        fn flatten_entries(into: &mut Vec<SelectRow>, entries: &[SelectEntry]) {
                            for entry in entries {
                                match entry {
                                    SelectEntry::Item(item) => into.push(SelectRow::Item(item.clone())),
                                    SelectEntry::Label(label) => into.push(SelectRow::Label(label.clone())),
                                    SelectEntry::Group(group) => flatten_entries(into, &group.entries),
                                    SelectEntry::Separator(_) => into.push(SelectRow::Separator),
                                }
                            }
                        }

                        let mut rows: Vec<SelectRow> = Vec::new();
                        flatten_entries(&mut rows, entries);

                        let item_count = rows
                            .iter()
                            .filter(|r| matches!(r, SelectRow::Item(_)))
                            .count();

                        let disabled: Vec<bool> = rows
                            .iter()
                            .map(|row| match row {
                                SelectRow::Item(item) => item.disabled || !enabled,
                                SelectRow::Label(_) | SelectRow::Separator => true,
                            })
                            .collect();

                        let labels: Vec<Arc<str>> = rows
                            .iter()
                            .map(|row| match row {
                                SelectRow::Item(item) => item.label.clone(),
                                SelectRow::Label(_) | SelectRow::Separator => Arc::from(""),
                            })
                            .collect();
                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());

                        let active = if let Some(selected) = selected.as_deref() {
                            let selected_idx = rows.iter().position(|row| match row {
                                SelectRow::Item(item) => item.value.as_ref() == selected,
                                SelectRow::Label(_) | SelectRow::Separator => false,
                            });
                            selected_idx
                                .and_then(|idx| (!disabled.get(idx).copied().unwrap_or(true)).then_some(idx))
                                .or_else(|| roving_focus::first_enabled(&disabled))
                        } else {
                            roving_focus::first_enabled(&disabled)
                        };
                        let roving = RovingFocusProps {
                            enabled: true,
                            wrap: loop_navigation,
                            disabled: Arc::from(disabled.clone().into_boxed_slice()),
                            ..Default::default()
                        };

                        let shadow = decl_style::shadow_sm(&theme_for_overlay, radius);
                        let arrow_bg = theme_for_overlay.colors.panel_background;
                        let arrow_border = border;

                        let wrapper =
                            popper_content::popper_wrapper_at(cx, placed, wrapper_insets, move |cx| {
                                let arrow_el = popper_arrow::diamond_arrow_element(
                                    cx,
                                    &layout,
                                    wrapper_insets,
                                    arrow_size,
                                    DiamondArrowStyle {
                                        bg: arrow_bg,
                                        border: Some(arrow_border),
                                        border_width,
                                    },
                                );

                                let panel = cx.container(
                                    ContainerProps {
                                        layout: popper_content::popper_panel_layout(
                                            placed,
                                            wrapper_insets,
                                            Overflow::Clip,
                                        ),
                                        padding: Edges::all(Px(0.0)),
                                        background: Some(theme_for_overlay.colors.panel_background),
                                        shadow: Some(shadow),
                                        border: Edges::all(border_width),
                                        border_color: Some(border),
                                        corner_radii: Corners::all(radius),
                                    },
                                    |cx| {
                                        vec![decl_scroll::overflow_scrollbar(
                                            cx,
                                            LayoutRefinement::default().w_full().h_full(),
                                            |cx| {
                                                vec![cx.semantics(
                                                    SemanticsProps {
                                                        layout: LayoutStyle::default(),
                                                        role: SemanticsRole::ListBox,
                                                        ..Default::default()
                                                    },
                                                    |cx| {
                                                        vec![cx.roving_flex(
                                                            RovingFlexProps {
                                                                flex: FlexProps {
                                                                    layout: LayoutStyle::default(),
                                                                    direction: fret_core::Axis::Vertical,
                                                                    gap: Px(0.0),
                                                                    padding: Edges::all(Px(4.0)),
                                                                    justify: MainAlign::Start,
                                                                    align: CrossAlign::Stretch,
                                                                    wrap: false,
                                                                },
                                                                roving,
                                                            },
                                                            |cx| {
                                                                cx.roving_nav_apg();
                                                                cx.roving_typeahead_prefix_arc_str(
                                                                    labels_arc.clone(),
                                                                    30,
                                                                );

                                                                let mut out = Vec::with_capacity(rows.len());
                                                                let mut item_ordinal: usize = 0;

                                                                for (row_idx, row) in rows.iter().cloned().enumerate() {
                                                                    match row {
                                                                        SelectRow::Label(label) => {
                                                                            let theme = Theme::global(&*cx.app).clone();
                                                                            let fg = theme
                                                                                .color_by_key("muted.foreground")
                                                                                .or_else(|| theme.color_by_key("muted-foreground"))
                                                                                .unwrap_or(theme.colors.text_muted);

                                                                            let mut layout = LayoutStyle::default();
                                                                            layout.size.width = Length::Fill;

                                                                            out.push(cx.text_props(TextProps {
                                                                                layout,
                                                                                text: label.text,
                                                                                style: Some(TextStyle {
                                                                                    font: FontId::default(),
                                                                                    size: theme.metrics.font_size,
                                                                                    weight: FontWeight::NORMAL,
                                                                                    line_height: Some(theme.metrics.font_line_height),
                                                                                    letter_spacing_em: None,
                                                                                }),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Ellipsis,
                                                                                color: Some(fg),
                                                                            }));
                                                                        }
                                                                        SelectRow::Separator => {
                                                                            let theme = Theme::global(&*cx.app).clone();
                                                                            let border = theme
                                                                                .color_by_key("border")
                                                                                .unwrap_or(theme.colors.panel_border);

                                                                            out.push(cx.container(
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
                                                                            ));
                                                                        }
                                                                        SelectRow::Item(item) => {
                                                                            let item_disabled =
                                                                                disabled.get(row_idx).copied().unwrap_or(true);
                                                                            let tab_stop = active.is_some_and(|a| a == row_idx);
                                                                            let is_selected = selected
                                                                                .as_ref()
                                                                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                                                            let item_ring = decl_style::focus_ring(
                                                                                &theme_for_overlay,
                                                                                theme_for_overlay.metrics.radius_sm,
                                                                            );

                                                                            let model = model.clone();
                                                                            let open = open_for_overlay.clone();
                                                                            let text_style = text_style_for_overlay.clone();

                                                                            let pos = item_ordinal;
                                                                            item_ordinal = item_ordinal.saturating_add(1);

                                                                            out.push(cx.pressable_with_id(
                                                                                PressableProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout.size.height = Length::Px(item_h);
                                                                                        layout
                                                                                    },
                                                                                    enabled: !item_disabled,
                                                                                    focusable: tab_stop,
                                                                                    focus_ring: Some(item_ring),
                                                                                    a11y: PressableA11y {
                                                                                        role: Some(SemanticsRole::ListBoxOption),
                                                                                        label: Some(item.label.clone()),
                                                                                        selected: is_selected,
                                                                                        ..Default::default()
                                                                                    }
                                                                                    .with_collection_position(pos, item_count),
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx, st, id| {
                                                                                    let _ = id;

                                                                                    cx.pressable_set_option_arc_str(
                                                                                        &model,
                                                                                        item.value.clone(),
                                                                                    );
                                                                                    cx.pressable_set_bool(&open, false);

                                                                                    let theme = Theme::global(&*cx.app).clone();
                                                                                    let mut bg = Color::TRANSPARENT;
                                                                                    if is_selected {
                                                                                        bg = alpha_mul(theme.colors.selection_background, 0.35);
                                                                                    }
                                                                                    if st.hovered || st.pressed {
                                                                                        bg = alpha_mul(theme.colors.selection_background, 0.45);
                                                                                    }
                                                                                    if st.focused {
                                                                                        bg = alpha_mul(theme.colors.selection_background, 0.45);
                                                                                    }

                                                                                    let fg = if item_disabled {
                                                                                        alpha_mul(fg_muted, 0.8)
                                                                                    } else {
                                                                                        fg
                                                                                    };

                                                                                    let icon = decl_icon::icon_with(
                                                                                        cx,
                                                                                        ids::ui::CHECK,
                                                                                        Some(Px(16.0)),
                                                                                        Some(ColorRef::Color(if item_disabled {
                                                                                            alpha_mul(fg_muted, 0.8)
                                                                                        } else {
                                                                                            fg
                                                                                        })),
                                                                                    );
                                                                                    let icon = cx.opacity(
                                                                                        if is_selected { 1.0 } else { 0.0 },
                                                                                        move |_cx| vec![icon],
                                                                                    );

                                                                                    vec![cx.container(
                                                                                        ContainerProps {
                                                                                            layout: {
                                                                                                let mut layout =
                                                                                                    LayoutStyle::default();
                                                                                                layout.size.width = Length::Fill;
                                                                                                layout.size.height = Length::Fill;
                                                                                                layout
                                                                                            },
                                                                                            padding: Edges::all(Px(8.0)),
                                                                                            background: Some(bg),
                                                                                            shadow: None,
                                                                                            border: Edges::all(Px(0.0)),
                                                                                            border_color: None,
                                                                                            corner_radii: Corners::all(theme.metrics.radius_sm),
                                                                                        },
                                                                                        |cx| {
                                                                                            vec![cx.flex(
                                                                                                FlexProps {
                                                                                                    layout: LayoutStyle::default(),
                                                                                                    direction: fret_core::Axis::Horizontal,
                                                                                                    gap: MetricRef::space(Space::N2)
                                                                                                        .resolve(&theme),
                                                                                                    padding: Edges::all(Px(0.0)),
                                                                                                    justify: MainAlign::SpaceBetween,
                                                                                                    align: CrossAlign::Center,
                                                                                                    wrap: false,
                                                                                                },
                                                                                                |cx| {
                                                                                                    vec![
                                                                                                        cx.text_props(TextProps {
                                                                                                            layout: LayoutStyle::default(),
                                                                                                            text: item.label.clone(),
                                                                                                            style: Some(text_style.clone()),
                                                                                                            wrap: TextWrap::None,
                                                                                                            overflow: TextOverflow::Ellipsis,
                                                                                                            color: Some(fg),
                                                                                                        }),
                                                                                                        icon,
                                                                                                    ]
                                                                                                },
                                                                                            )]
                                                                                        },
                                                                                    )]
                                                                                },
                                                                            ));
                                                                        }
                                                                    }
                                                                }

                                                                out
                                                            },
                                                        )]
                                                    },
                                                )]
                                            },
                                        )]
                                    },
                                );

                                if let Some(arrow_el) = arrow_el {
                                    vec![arrow_el, panel]
                                } else {
                                    vec![panel]
                                }
                            });

                        vec![wrapper]
                    });

                    let mut request = OverlayRequest::dismissible_popover(
                        trigger_id,
                        trigger_id,
                        open,
                        OverlayPresence::instant(true),
                        overlay_children,
                    );
                    request.root_name = Some(overlay_root_name);
                    OverlayController::request(cx, request);
            }

            let chrome = input_chrome_container_props(
                {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                resolved,
                border_color,
            );

            let content = move |cx: &mut ElementContext<'_, H>| {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: MetricRef::space(Space::N2).resolve(&theme),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            cx.text_props(TextProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout
                                },
                                text: label,
                                style: Some(text_style.clone()),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                                color: Some(if selected.is_some() { fg } else { fg_muted }),
                            }),
                            decl_icon::icon_with(cx, ids::ui::CHEVRON_DOWN, Some(Px(16.0)), None),
                        ]
                    },
                )]
            };

            (props, chrome, content)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics,
    };
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_core::{TextStyle, UiServices};
    use fret_runtime::{Effect, FrameId};
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
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![Select::new(model, open).items(items).into_element(cx)]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_arrow(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
        arrow: bool,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![
                    Select::new(model, open)
                        .items(items)
                        .arrow(arrow)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        entries: Vec<SelectEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![Select::new(model, open).entries(entries).into_element(cx)]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn select_popover_items_have_collection_position_metadata() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the popover and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }

    #[test]
    fn select_trigger_enter_opens_on_key_down_and_does_not_toggle_closed_on_key_up() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![SelectItem::new("alpha", "Alpha"), SelectItem::new("beta", "Beta")];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open.clone(),
            items,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_trigger_typeahead_updates_selection_without_opening() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyB,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(!app.models().get_copied(&open).unwrap_or(false));
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if *after == Duration::from_millis(500) => {
                    Some(*token)
                }
                _ => None,
            })
            .expect("typeahead clear timer token");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(!app.models().get_copied(&open).unwrap_or(false));

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_label_and_separator_do_not_affect_positions_or_initial_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![
            SelectEntry::Label(SelectLabel::new("Fruits")),
            SelectEntry::Item(SelectItem::new("alpha", "Alpha")),
            SelectEntry::Separator(SelectSeparator),
            SelectEntry::Item(SelectItem::new("beta", "Beta")),
        ];

        let _ = render_frame_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(focused_node.label.as_deref(), Some("Beta"));

        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(2));
    }

    #[test]
    fn select_roving_navigation_does_not_commit_value_until_activation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        assert!(
            ui.focus().is_some(),
            "expected focus to move into the open select"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
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
            open.clone(),
            items,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_arrow_is_hit_testable_and_does_not_dismiss_on_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_with_arrow(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
            true,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("list node");
        let list_bounds = ui.debug_node_bounds(list.id).expect("list bounds");

        let click = Point::new(
            Px(list_bounds.origin.x.0 + list_bounds.size.width.0 * 0.5),
            Px(list_bounds.origin.y.0 - 1.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }
}
