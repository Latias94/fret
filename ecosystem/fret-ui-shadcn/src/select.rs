use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, Rect, SemanticsRole, TextOverflow,
    TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    OpacityProps, Overflow, PointerRegionProps, PositionStyle, PressableA11y, PressableProps,
    ScrollProps, SizeStyle, StackProps, TextProps, VisualTransformProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome as decl_chrome;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::overlay_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::roving_focus;
use fret_ui_kit::headless::select_item_aligned as item_aligned;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::active_descendant as active_desc;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::select as radix_select;
use fret_ui_kit::recipes::input::{
    InputTokenKeys, input_chrome_container_props, resolve_input_chrome,
};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Space,
};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn select_scroll_with_buttons<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: Theme,
    item_step: Px,
    initial_scroll_to_y: Option<Px>,
    viewport_id_out: &Cell<Option<GlobalElementId>>,
    content: impl FnOnce(&mut ElementContext<'_, H>, &Cell<Option<GlobalElementId>>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            },
            direction: fret_core::Axis::Vertical,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
            if let Some(y) = initial_scroll_to_y {
                let prev = handle.offset();
                handle.scroll_to_offset(Point::new(prev.x, y));
            }

            let scroll_button_h = theme
                .metric_by_key("component.select.scroll_button_height")
                .unwrap_or(Px(24.0));

            let max = handle.max_offset();
            let offset = handle.offset();
            // Guard against fractional max offsets (layout rounding) causing scroll affordances to
            // appear when content visually fits.
            let scroll_epsilon = Px(0.5);
            let has_scroll = max.y.0 > scroll_epsilon.0;
            let show_up = has_scroll && offset.y.0 > scroll_epsilon.0;
            let show_down = has_scroll && (offset.y.0 + scroll_epsilon.0) < max.y.0;

            let scroll_button = |cx: &mut ElementContext<'_, H>,
                                 icon: fret_icons::IconId,
                                 label: &'static str,
                                 dir: f32| {
                let handle = handle.clone();
                let theme = theme.clone();
                cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(scroll_button_h);
                            layout
                        },
                        enabled: true,
                        focusable: false,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from(label)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                            let prev = handle.offset();
                            let next = Point::new(prev.x, Px(prev.y.0 + item_step.0 * dir));
                            handle.scroll_to_offset(next);
                            host.request_redraw(action_cx.window);
                        }));

                        vec![cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                // new-york-v4: `py-1` and no hover fill.
                                padding: Edges {
                                    top: Px(4.0),
                                    right: Px(0.0),
                                    bottom: Px(4.0),
                                    left: Px(0.0),
                                },
                                background: Some(Color::TRANSPARENT),
                                shadow: None,
                                border: Edges::all(Px(0.0)),
                                border_color: None,
                                corner_radii: Corners::all(Px(0.0)),
                            },
                            |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Fill;
                                            layout
                                        },
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(0.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::Center,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    |cx| {
                                        vec![decl_icon::icon_with(
                                            cx,
                                            icon,
                                            Some(Px(16.0)),
                                            Some(ColorRef::Color(
                                                theme
                                                    .color_by_key("muted-foreground")
                                                    .unwrap_or(theme.colors.text_muted),
                                            )),
                                        )]
                                    },
                                )]
                            },
                        )]
                    },
                )
            };

            let handle_for_stack = handle.clone();
            let stack = cx.stack_props(
                StackProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Px(0.0));
                        layout.flex.grow = 1.0;
                        layout.flex.shrink = 1.0;
                        layout.flex.basis = Length::Px(Px(0.0));
                        layout
                    },
                },
                move |cx| {
                    let active_element = Cell::new(None::<GlobalElementId>);
                    let active_element_ref = &active_element;

                    let mut scroll_layout = LayoutStyle::default();
                    scroll_layout.size.width = Length::Fill;
                    scroll_layout.size.height = Length::Fill;
                    scroll_layout.overflow = Overflow::Clip;

                    let scroll = cx.scroll(
                        ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(handle_for_stack.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;
                                        layout
                                    },
                                    // new-york-v4: `SelectPrimitive.Viewport` uses `p-1`.
                                    padding: Edges::all(Px(4.0)),
                                    ..Default::default()
                                },
                                move |cx| content(cx, active_element_ref),
                            )]
                        },
                    );
                    viewport_id_out.set(Some(scroll.id));

                    if let Some(active_element) = active_element.get() {
                        let _ = active_desc::scroll_active_element_into_view_y(
                            cx,
                            &handle_for_stack,
                            scroll.id,
                            active_element,
                        );
                    }

                    vec![scroll]
                },
            );

            let mut out = Vec::new();
            if show_up {
                out.push(scroll_button(cx, ids::ui::CHEVRON_UP, "Scroll up", -1.0));
            }
            out.push(stack);
            if show_down {
                out.push(scroll_button(cx, ids::ui::CHEVRON_DOWN, "Scroll down", 1.0));
            }
            out
        },
    )
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

/// Matches Radix Select `position`: item-aligned (default upstream) vs popper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectPosition {
    ItemAligned,
    #[default]
    Popper,
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
    position: SelectPosition,
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
            position: SelectPosition::default(),
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

    pub fn position(mut self, position: SelectPosition) -> Self {
        self.position = position;
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
            self.position,
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
        SelectPosition::default(),
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
    position: SelectPosition,
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
        let motion = OverlayController::transition_with_durations_and_easing(
            cx,
            is_open,
            overlay_motion::SHADCN_MOTION_TICKS_100,
            overlay_motion::SHADCN_MOTION_TICKS_100,
            overlay_motion::shadcn_ease,
        );
        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: is_open,
        };
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
            trigger: radix_select::SelectTriggerKeyState,
            pointer: radix_select::SelectTriggerPointerState,
            content: radix_select::SelectContentKeyState,
            was_open: bool,
            value_node: Option<GlobalElementId>,
            viewport: Option<GlobalElementId>,
            listbox: Option<GlobalElementId>,
            content_panel: Option<GlobalElementId>,
            selected_item: Option<GlobalElementId>,
            selected_item_text: Option<GlobalElementId>,
            pending_item_aligned_scroll_to_y: Option<Px>,
            did_item_aligned_scroll: bool,
        }

        impl SelectTriggerKeyState {
            fn new() -> Self {
                Self {
                    trigger: radix_select::SelectTriggerKeyState::default(),
                    pointer: radix_select::SelectTriggerPointerState::default(),
                    content: radix_select::SelectContentKeyState::default(),
                    was_open: false,
                    value_node: None,
                    viewport: None,
                    listbox: None,
                    content_panel: None,
                    selected_item: None,
                    selected_item_text: None,
                    pending_item_aligned_scroll_to_y: None,
                    did_item_aligned_scroll: false,
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

        // `control_chrome_pressable_with_id_props` stores handlers; avoid moving `open` into the
        // trigger closure so it can still be used for pointer handling later.
        let open_for_trigger = open.clone();

        let trigger = decl_chrome::control_chrome_pressable_with_id_props(cx, move |cx, st, trigger_id| {
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
                    state.trigger.on_timer(token) || state.content.on_timer(token)
                }),
            );

            let open_for_key = open_for_trigger.clone();
            let model_for_key = model.clone();
            let values_for_key = typeahead_values.clone();
            let labels_for_key = typeahead_labels.clone();
            let disabled_for_key = typeahead_disabled.clone();
            let state_for_key = trigger_state.clone();
            cx.key_on_key_down_for(
                trigger_id,
                Arc::new(move |host, action_cx, it| {
                    let mut state = state_for_key
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    state.trigger.handle_key_down_when_closed(
                        host,
                        action_cx.window,
                        &open_for_key,
                        &model_for_key,
                        values_for_key.as_ref(),
                        labels_for_key.as_ref(),
                        disabled_for_key.as_ref(),
                        it.key,
                        it.modifiers,
                        it.repeat,
                    )
                }),
            );

            let open_for_activate = open_for_trigger.clone();
            let state_for_activate = trigger_state.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut state = state_for_activate
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());

                if state.trigger.take_suppress_next_activate() {
                    return;
                }

                state.trigger.clear_typeahead(host);

                let _ = host.models_mut().update(&open_for_activate, |v| *v = true);
                host.request_redraw(action_cx.window);
            }));

            let border_color = if st.hovered || st.pressed || st.focused {
                alpha_mul(border_focus, 0.85)
            } else {
                border
            };

            let mut props = PressableProps {
                layout: trigger_layout,
                enabled,
                focusable: enabled,
                focus_ring: Some(ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ComboBox),
                    label: a11y_label.clone(),
                    expanded: Some(is_open),
                    controls_element: None,
                    ..Default::default()
                },
                ..Default::default()
            };

            // Radix Select uses `hideOthers(content)` (aria-hide outside) and disables outside
            // pointer events while open. In Fret we approximate that by installing a modal barrier
            // layer (blocks underlay input + gates accessibility roots) even though the content
            // itself remains `role=listbox` (not a dialog).
            let overlay_root_name = radix_select::select_root_name(trigger_id);
            let listbox_controls_element: Cell<Option<u64>> = Cell::new(None);
            let listbox_controls_element = &listbox_controls_element;

            if motion.present
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
                    let desired_h = Px(
                        (item_h.0 * item_len as f32)
                            .min(max_h.0)
                            .max(item_h.0)
                            .min(outer.size.height.0),
                    );
                    let desired_w = Px(anchor.size.width.0.max(min_width.0).min(outer.size.width.0));
                    let desired = fret_core::Size::new(desired_w, desired_h);

                    let border_width = resolved.border_width;

                    let mut arrow_layout = None;
                    let mut wrapper_insets = Edges::all(Px(0.0));
                    let mut motion_side: Side = side.into();

                    let mut item_aligned_rect: Option<Rect> = None;
                    if position == SelectPosition::ItemAligned {
                        let (value_node, viewport, listbox, content_panel, selected_item, selected_item_text, did_scroll) =
                            {
                                let state = trigger_state
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                (
                                    state.value_node,
                                    state.viewport,
                                    state.listbox,
                                    state.content_panel,
                                    state.selected_item,
                                    state.selected_item_text,
                                    state.did_item_aligned_scroll,
                                )
                            };

                        if let (
                            Some(value_node),
                            Some(viewport),
                            Some(listbox),
                            Some(content_panel),
                            Some(selected_item),
                            Some(selected_item_text),
                        ) = (
                            value_node,
                            viewport,
                            listbox,
                            content_panel,
                            selected_item,
                            selected_item_text,
                        ) {
                            if let (
                                Some(value_node),
                                Some(viewport),
                                Some(listbox),
                                Some(content),
                                Some(selected_item),
                                Some(selected_item_text),
                            ) = (
                                overlay::anchor_bounds_for_element(cx, value_node),
                                overlay::anchor_bounds_for_element(cx, viewport),
                                overlay::anchor_bounds_for_element(cx, listbox),
                                overlay::anchor_bounds_for_element(cx, content_panel),
                                overlay::anchor_bounds_for_element(cx, selected_item),
                                overlay::anchor_bounds_for_element(cx, selected_item_text),
                            ) {
                                let out = item_aligned::select_item_aligned_position(
                                    item_aligned::SelectItemAlignedInputs {
                                        direction: LayoutDirection::Ltr,
                                        window: cx.bounds,
                                        trigger: anchor,
                                        content,
                                        value_node,
                                        selected_item_text,
                                        selected_item,
                                        viewport,
                                        content_border_top: border_width,
                                        content_padding_top: Px(0.0),
                                        content_border_bottom: border_width,
                                        content_padding_bottom: Px(0.0),
                                        viewport_padding_top: Px(4.0),
                                        viewport_padding_bottom: Px(4.0),
                                        items_height: listbox.size.height,
                                    },
                                );

                                if let Some(scroll_to) = out.scroll_to_y
                                    && !did_scroll
                                {
                                    let mut state = trigger_state
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    state.pending_item_aligned_scroll_to_y = Some(scroll_to);
                                    state.did_item_aligned_scroll = true;
                                }

                                let margin = item_aligned::SELECT_ITEM_ALIGNED_CONTENT_MARGIN;
                                let inset_left = out.left.unwrap_or(margin);
                                let inset_top = if out.top.is_some() {
                                    margin
                                } else if out.bottom.is_some() {
                                    Px(cx.bounds.size.height.0 - margin.0 - out.height.0)
                                } else {
                                    margin
                                };

                                let placed = Rect::new(
                                    Point::new(inset_left, inset_top),
                                    fret_core::Size::new(out.width, out.height),
                                );
                                motion_side = if placed.origin.y.0 >= anchor.origin.y.0 {
                                    Side::Bottom
                                } else {
                                    Side::Top
                                };
                                item_aligned_rect = Some(placed);
                            }
                        }
                    }

                    let placed = if let Some(placed) = item_aligned_rect {
                        placed
                    } else {
                        let side_offset = side_offset_override.unwrap_or_else(|| {
                            theme
                                .metric_by_key("component.select.popover_offset")
                                .unwrap_or(Px(6.0))
                        });

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
                        wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                        arrow_layout = Some((layout, arrow_protrusion));
                        layout.rect
                    };

                    let origin = overlay_motion::shadcn_transform_origin_for_anchored_rect(
                        anchor,
                        placed,
                        motion_side,
                    );
                    let zoom = overlay_motion::shadcn_zoom_transform(origin, motion.progress);
                    let slide =
                        overlay_motion::shadcn_enter_slide_transform(motion_side, motion.progress, is_open);
                    let transform = slide * zoom;
                    let opacity = motion.progress;

                    let theme_for_overlay = theme.clone();
                    let text_style_for_overlay = text_style.clone();
                    let open_for_overlay = open_for_trigger.clone();
                    let trigger_state_for_overlay = trigger_state.clone();
                    let list_focus_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let list_focus_id_out = &list_focus_id_out_cell;
                    let viewport_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let viewport_id_out = &viewport_id_out_cell;
                    let content_panel_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let content_panel_id_out = &content_panel_id_out_cell;
                    let selected_item_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let selected_item_id_out = &selected_item_id_out_cell;
                    let selected_item_text_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let selected_item_text_id_out = &selected_item_text_id_out_cell;
                    let arrow_layout_for_children = arrow_layout.clone();
                    let trigger_state_for_overlay_for_children = trigger_state_for_overlay.clone();

                    let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                        let trigger_state_for_overlay = trigger_state_for_overlay_for_children.clone();
                        let barrier_layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                top: Some(Px(0.0)),
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let open_for_barrier = open_for_overlay.clone();
                        let barrier = cx.pressable(
                            PressableProps {
                                layout: barrier_layout,
                                enabled: true,
                                focusable: false,
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_set_bool(&open_for_barrier, false);
                                Vec::new()
                            },
                        );

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

                        let initial_active_row = if let Some(selected) = selected.as_deref() {
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
                        let active_row = {
                            let mut state = trigger_state
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());

                            if is_open {
                                if !state.was_open {
                                    state.was_open = true;
                                    state.content.reset_on_open(initial_active_row);
                                    state.trigger.reset_typeahead_buffer();
                            } else if state.content.active_row().is_none() {
                                state.content.set_active_row(initial_active_row);
                            }
                        } else {
                            state.was_open = false;
                            state.content.set_active_row(None);
                        }

                        state.content.active_row()
                    };

                        let shadow = decl_style::shadow_md(&theme_for_overlay, radius);
                        let arrow_bg = theme_for_overlay.colors.panel_background;
                        let arrow_border = border;
                        let initial_scroll_to_y = {
                            let mut state = trigger_state_for_overlay
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            state.pending_item_aligned_scroll_to_y.take()
                        };

                        let trigger_state_for_overlay_in_content = trigger_state_for_overlay.clone();
                        let content = popper_content::popper_wrapper_at(cx, placed, wrapper_insets, move |cx| {
                                let arrow_el = arrow_layout_for_children.as_ref().and_then(|(layout, _)| {
                                    popper_arrow::diamond_arrow_element(
                                        cx,
                                        layout,
                                        wrapper_insets,
                                        arrow_size,
                                        DiamondArrowStyle {
                                            bg: arrow_bg,
                                            border: Some(arrow_border),
                                            border_width,
                                        },
                                    )
                                });

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
                                    vec![select_scroll_with_buttons(
                                        cx,
                                        theme_for_overlay.clone(),
                                        item_h,
                                        initial_scroll_to_y,
                                        viewport_id_out,
                                        move |cx, active_element| {
                                            let disabled_for_key: Arc<[bool]> =
                                                Arc::from(disabled.clone().into_boxed_slice());
                                            let labels_for_key = labels_arc.clone();
                                            let values_by_row: Arc<[Option<Arc<str>>]> = Arc::from(
                                                rows.iter()
                                                    .map(|row| match row {
                                                        SelectRow::Item(item) => Some(item.value.clone()),
                                                        SelectRow::Label(_) | SelectRow::Separator => None,
                                                    })
                                                    .collect::<Vec<_>>()
                                                    .into_boxed_slice(),
                                            );

                                            let state_for_key =
                                                trigger_state_for_overlay_in_content.clone();
                                            let open_for_key = open_for_overlay.clone();
                                            let model_for_key = model.clone();
                                            let loop_navigation_for_key = loop_navigation;

                                            vec![cx.pressable_with_id_props(move |cx, _st, listbox_id| {
                                                list_focus_id_out.set(Some(listbox_id));
                                                listbox_controls_element.set(Some(listbox_id.0));

                                                cx.key_on_key_down_for(
                                                    listbox_id,
                                                    Arc::new(move |host, action_cx, it| {
                                                        let mut state = state_for_key
                                                            .lock()
                                                            .unwrap_or_else(|e| e.into_inner());
                                                        state.content.handle_key_down_when_open(
                                                            host,
                                                            action_cx.window,
                                                            &open_for_key,
                                                            &model_for_key,
                                                            values_by_row.as_ref(),
                                                            labels_for_key.as_ref(),
                                                            disabled_for_key.as_ref(),
                                                            it.key,
                                                            it.repeat,
                                                            loop_navigation_for_key,
                                                        )
                                                    }),
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

                                                                            let label_text_px = Px(
                                                                                (theme.metrics.font_size.0 - 2.0)
                                                                                    .max(10.0),
                                                                            );
                                                                            let label_line_height = Px(
                                                                                (theme
                                                                                    .metrics
                                                                                    .font_line_height
                                                                                    .0
                                                                                    - 4.0)
                                                                                    .max(12.0),
                                                                            );

                                                                            out.push(cx.container(
                                                                                ContainerProps {
                                                                                    layout: {
                                                                                        let mut layout =
                                                                                            LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout
                                                                                    },
                                                                                    // new-york-v4: `px-2 py-1.5`
                                                                                    padding: Edges {
                                                                                        top: Px(6.0),
                                                                                        right: Px(8.0),
                                                                                        bottom: Px(6.0),
                                                                                        left: Px(8.0),
                                                                                    },
                                                                                    background: None,
                                                                                    shadow: None,
                                                                                    border: Edges::all(Px(0.0)),
                                                                                    border_color: None,
                                                                                    corner_radii: Corners::all(Px(0.0)),
                                                                                },
                                                                                move |cx| {
                                                                                    let mut layout =
                                                                                        LayoutStyle::default();
                                                                                    layout.size.width = Length::Fill;
                                                                                    vec![cx.text_props(TextProps {
                                                                                        layout,
                                                                                        text: label.text,
                                                                                        style: Some(TextStyle {
                                                                                            font: FontId::default(),
                                                                                            size: label_text_px,
                                                                                            weight: FontWeight::NORMAL,
                                                                                            line_height: Some(
                                                                                                label_line_height,
                                                                                            ),
                                                                                            letter_spacing_em: None,
                                                                                        }),
                                                                                        wrap: TextWrap::None,
                                                                                        overflow: TextOverflow::Clip,
                                                                                        color: Some(fg),
                                                                                    })]
                                                                                },
                                                                            ));
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
                                                                                        // new-york-v4: `SelectSeparator` uses `-mx-1 my-1`.
                                                                                        layout.margin.left =
                                                                                            fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                        layout.margin.right =
                                                                                            fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                        layout.margin.top =
                                                                                            fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                                        layout.margin.bottom =
                                                                                            fret_ui::element::MarginEdge::Px(Px(4.0));
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
                                                                            let is_active =
                                                                                active_row.is_some_and(|a| a == row_idx);
                                                                            let is_selected = selected
                                                                                .as_ref()
                                                                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                                                            let model = model.clone();
                                                                            let open = open_for_overlay.clone();
                                                                            let text_style = text_style_for_overlay.clone();

                                                                            let pos = item_ordinal;
                                                                            item_ordinal = item_ordinal.saturating_add(1);
                                                                            let state_for_hover =
                                                                                trigger_state_for_overlay_in_content
                                                                                    .clone();
                                                                            let row_idx_for_hover = row_idx;

                                                                            out.push(cx.pressable_with_id(
                                                                                PressableProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout.size.height = Length::Px(item_h);
                                                                                        layout
                                                                                    },
                                                                                    enabled: !item_disabled,
                                                                                    focusable: false,
                                                                                    focus_ring: None,
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
                                                                                    if is_selected {
                                                                                        selected_item_id_out.set(Some(id));
                                                                                    }
                                                                                    if is_active {
                                                                                        active_element.set(Some(id));
                                                                                    }

                                                                                    cx.pressable_set_option_arc_str(
                                                                                        &model,
                                                                                        item.value.clone(),
                                                                                    );
                                                                                    cx.pressable_set_bool(&open, false);

                                                                                    if !item_disabled {
                                                                                        cx.pressable_add_on_hover_change(Arc::new(
                                                                                            move |host, action_cx, hovered| {
                                                                                                if !hovered {
                                                                                                    return;
                                                                                                }
                                                                                                let mut state = state_for_hover
                                                                                                    .lock()
                                                                                                    .unwrap_or_else(|e| e.into_inner());
                                                                                                if state.content.active_row()
                                                                                                    != Some(row_idx_for_hover)
                                                                                                {
                                                                                                    state.content.set_active_row(
                                                                                                        Some(row_idx_for_hover),
                                                                                                    );
                                                                                                    host.request_redraw(
                                                                                                        action_cx.window,
                                                                                                    );
                                                                                                }
                                                                                            },
                                                                                        ));
                                                                                    }

                                                                                    let theme = Theme::global(&*cx.app).clone();
                                                                                    // new-york-v4: items highlight on focus/hover via `bg-accent`.
                                                                                    let bg_accent = theme
                                                                                        .color_by_key("accent")
                                                                                        .or_else(|| theme.color_by_key("accent.background"))
                                                                                        .unwrap_or(theme.colors.hover_background);
                                                                                    let fg_accent = theme
                                                                                        .color_by_key("accent-foreground")
                                                                                        .or_else(|| theme.color_by_key("accent.foreground"))
                                                                                        .unwrap_or(theme.colors.text_primary);

                                                                                    let mut bg = Color::TRANSPARENT;
                                                                                    let mut fg = if item_disabled {
                                                                                        alpha_mul(fg_muted, 0.8)
                                                                                    } else {
                                                                                        fg
                                                                                    };
                                                                                    if is_active || st.hovered || st.pressed {
                                                                                        bg = bg_accent;
                                                                                        fg = fg_accent;
                                                                                    }

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
                                                                                            // new-york-v4: `py-1.5 pl-2 pr-8`
                                                                                            padding: Edges {
                                                                                                top: Px(6.0),
                                                                                                right: Px(32.0),
                                                                                                bottom: Px(6.0),
                                                                                                left: Px(8.0),
                                                                                            },
                                                                                            background: Some(bg),
                                                                                            shadow: None,
                                                                                            border: Edges::all(Px(0.0)),
                                                                                            border_color: None,
                                                                                            corner_radii: Corners::all(theme.metrics.radius_sm),
                                                                                        },
                                                                                        |cx| {
                                                                                            let text = cx.container(
                                                                                                ContainerProps {
                                                                                                    layout: {
                                                                                                        let mut layout =
                                                                                                            LayoutStyle::default();
                                                                                                        layout.size.width =
                                                                                                            Length::Fill;
                                                                                                        layout
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                |cx| {
                                                                                                    vec![cx.text_props(TextProps {
                                                                                                        layout: {
                                                                                                            let mut layout =
                                                                                                                LayoutStyle::default();
                                                                                                            layout.size.width =
                                                                                                                Length::Fill;
                                                                                                            layout
                                                                                                        },
                                                                                                        text: item.label.clone(),
                                                                                                        style: Some(text_style.clone()),
                                                                                                        wrap: TextWrap::None,
                                                                                                        overflow: TextOverflow::Clip,
                                                                                                        color: Some(fg),
                                                                                                    })]
                                                                                                },
                                                                                            );
                                                                                            if is_selected {
                                                                                                selected_item_text_id_out
                                                                                                    .set(Some(text.id));
                                                                                            }

                                                                                            // Indicator slot matches upstream: absolute at the end, but reserve `pr-8`.
                                                                                            let indicator_size = Px(14.0);
                                                                                            let indicator_top = Px(
                                                                                                ((item_h.0 - indicator_size.0)
                                                                                                    * 0.5)
                                                                                                    .max(0.0),
                                                                                            );
                                                                                            let indicator = cx.container(
                                                                                                ContainerProps {
                                                                                                    layout: LayoutStyle {
                                                                                                        position: PositionStyle::Absolute,
                                                                                                        inset: InsetStyle {
                                                                                                            top: Some(indicator_top),
                                                                                                            right: Some(Px(8.0)),
                                                                                                            bottom: None,
                                                                                                            left: None,
                                                                                                        },
                                                                                                        size: SizeStyle {
                                                                                                            width: Length::Px(
                                                                                                                indicator_size,
                                                                                                            ),
                                                                                                            height: Length::Px(
                                                                                                                indicator_size,
                                                                                                            ),
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    padding: Edges::all(Px(0.0)),
                                                                                                    background: None,
                                                                                                    shadow: None,
                                                                                                    border: Edges::all(Px(0.0)),
                                                                                                    border_color: None,
                                                                                                    corner_radii: Corners::all(Px(0.0)),
                                                                                                },
                                                                                                |cx| {
                                                                                                    vec![cx.flex(
                                                                                                        FlexProps {
                                                                                                            layout: {
                                                                                                                let mut layout =
                                                                                                                    LayoutStyle::default();
                                                                                                                layout.size.width =
                                                                                                                    Length::Fill;
                                                                                                                layout.size.height =
                                                                                                                    Length::Fill;
                                                                                                                layout
                                                                                                            },
                                                                                                            direction: fret_core::Axis::Horizontal,
                                                                                                            gap: Px(0.0),
                                                                                                            padding: Edges::all(Px(0.0)),
                                                                                                            justify: MainAlign::Center,
                                                                                                            align: CrossAlign::Center,
                                                                                                            wrap: false,
                                                                                                        },
                                                                                                        |_cx| vec![icon.clone()],
                                                                                                    )]
                                                                                                },
                                                                                            );

                                                                                            vec![cx.stack_props(
                                                                                                StackProps {
                                                                                                    layout: {
                                                                                                        let mut layout =
                                                                                                            LayoutStyle::default();
                                                                                                        layout.size.width =
                                                                                                            Length::Fill;
                                                                                                        layout.size.height =
                                                                                                            Length::Fill;
                                                                                                        layout
                                                                                                    },
                                                                                                },
                                                                                                |_cx| vec![text, indicator],
                                                                                            )]
                                                                                        },
                                                                                    )]
                                                                                },
                                                                            ));
                                                                        }
                                                                    }
                                                                }

                                                                let active_descendant = active_element
                                                                    .get()
                                                                    .and_then(|id| cx.node_for_element(id));

                                                                (
                                                                    PressableProps {
                                                                        layout: {
                                                                            let mut layout = LayoutStyle::default();
                                                                            layout.size.width = Length::Fill;
                                                                            layout
                                                                        },
                                                                        enabled: true,
                                                                        focusable: true,
                                                                        focus_ring: None,
                                                                        a11y: PressableA11y {
                                                                            role: Some(SemanticsRole::ListBox),
                                                                            active_descendant,
                                                                            labelled_by_element: Some(trigger_id.0),
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    vec![cx.flex(
                                                                        FlexProps {
                                                                            layout: LayoutStyle::default(),
                                                                            direction: fret_core::Axis::Vertical,
                                                                            gap: Px(0.0),
                                                                            padding: Edges::all(Px(4.0)),
                                                                            justify: MainAlign::Start,
                                                                            align: CrossAlign::Stretch,
                                                                            wrap: false,
                                                                        },
                                                                        |_cx| out,
                                                                    )],
                                                                )
                                                            })]
                                        },
                                    )]
                                },
                            );

                                content_panel_id_out.set(Some(panel.id));

                                if let Some(arrow_el) = arrow_el {
                                    vec![arrow_el, panel]
                                } else {
                                    vec![panel]
                                }
                            });

                        let opacity_layout = LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        let animated = cx.opacity_props(
                            OpacityProps {
                                layout: opacity_layout.clone(),
                                opacity,
                            },
                            move |cx| {
                                vec![cx.visual_transform_props(
                                    VisualTransformProps {
                                        layout: opacity_layout,
                                        transform,
                                    },
                                    move |_cx| vec![content],
                                )]
                            },
                        );

                        {
                            let mut state = trigger_state_for_overlay
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            state.viewport = viewport_id_out.get();
                            state.listbox = list_focus_id_out.get();
                            state.content_panel = content_panel_id_out.get();
                            state.selected_item = selected_item_id_out.get();
                            state.selected_item_text = selected_item_text_id_out.get();
                            if !is_open {
                                state.did_item_aligned_scroll = false;
                            }
                        }

                        vec![barrier, animated]
                    });

                    let mut request = radix_select::modal_select_request(
                        trigger_id,
                        trigger_id,
                        open_for_trigger.clone(),
                        overlay_presence,
                        overlay_children,
                    );
                    request.initial_focus = list_focus_id_out.get();
                    radix_select::request_select(cx, request);
            }

            props.a11y.controls_element = listbox_controls_element.get();
            let chrome = input_chrome_container_props(
                {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                resolved,
                border_color,
            );

            let state_for_value_node = trigger_state.clone();

            let content = move |cx: &mut ElementContext<'_, H>| {
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: MetricRef::space(Space::N2).resolve(&theme),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            {
                                let layout = {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.min_width = Some(Px(0.0));
                                    layout.flex.grow = 1.0;
                                    layout.flex.shrink = 1.0;
                                    layout.flex.basis = Length::Px(Px(0.0));
                                    layout
                                };

                                let value_node = cx.container(
                                    ContainerProps {
                                        layout,
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.text_props(TextProps {
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
                                        })]
                                    },
                                );

                                let mut state = state_for_value_node
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                state.value_node = Some(value_node.id);

                                value_node
                            },
                                                        cx.opacity(0.5, |cx| {
                                vec![decl_icon::icon_with(
                                    cx,
                                    ids::ui::CHEVRON_DOWN,
                                    Some(Px(16.0)),
                                    Some(ColorRef::Color(fg_muted)),
                                )]
                            }),
                        ]
                    },
                )]
            };

            (props, chrome, content)
        });

        let trigger_id = trigger.id;
        let trigger_state: Arc<Mutex<SelectTriggerKeyState>> = cx.with_state_for(
            trigger_id,
            || Arc::new(Mutex::new(SelectTriggerKeyState::new())),
            |s| s.clone(),
        );

        let open_for_pointer_down = open.clone();
        let open_for_pointer_up = open.clone();
        let enabled_for_pointer = enabled;
        let state_for_pointer_down = trigger_state.clone();
        let state_for_pointer_move = trigger_state.clone();
        let state_for_pointer_up = trigger_state.clone();

        cx.pointer_region(PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(
                move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, down: PointerDownCx| {
                    let mut state = state_for_pointer_down
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    let handled = state.pointer.handle_pointer_down(
                        host,
                        action_cx,
                        down,
                        &open_for_pointer_down,
                        enabled_for_pointer,
                    );
                    if handled && matches!(down.pointer_type, fret_core::PointerType::Mouse | fret_core::PointerType::Unknown) {
                        state.trigger.clear_typeahead(host);
                    }
                    handled
                },
            ));

            cx.pointer_region_on_pointer_move(Arc::new(
                move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, mv: PointerMoveCx| {
                    let mut state = state_for_pointer_move
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    state.pointer.handle_pointer_move(host, action_cx, mv)
                },
            ));

            cx.pointer_region_on_pointer_up(Arc::new(
                move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, up: PointerUpCx| {
                    let was_open = host.models_mut().get_copied(&open_for_pointer_up).unwrap_or(false);

                    let mut state = state_for_pointer_up
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    let handled = state.pointer.handle_pointer_up(
                        host,
                        action_cx,
                        up,
                        &open_for_pointer_up,
                        enabled_for_pointer,
                    );

                    if handled
                        && !was_open
                        && host.models_mut().get_copied(&open_for_pointer_up).unwrap_or(false)
                        && matches!(up.pointer_type, fret_core::PointerType::Touch | fret_core::PointerType::Pen)
                    {
                        state.trigger.clear_typeahead(host);
                    }

                    handled
                },
            ));

            vec![trigger]
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

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ];

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
                Effect::SetTimer { token, after, .. }
                    if *after
                        == Duration::from_millis(
                            radix_select::SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS,
                        ) =>
                {
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
            entries.clone(),
        );
        // Third frame: allow `active_descendant` to resolve via last-frame node IDs.
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
        assert_eq!(focused_node.role, SemanticsRole::ListBox);

        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.flags.expanded)
            .expect("select trigger node");
        assert!(
            focused_node.labelled_by.iter().any(|id| *id == trigger.id),
            "listbox should be labelled by the trigger"
        );
        assert!(
            trigger.controls.iter().any(|id| *id == focused_node.id),
            "trigger should control the listbox"
        );

        let active = focused_node
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
    }

    #[test]
    fn select_open_installs_modal_barrier_root_for_a11y_isolation() {
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
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected select to install a modal barrier root");
        assert!(
            snap.roots
                .iter()
                .any(|r| r.root == barrier_root && r.blocks_underlay_input),
            "expected barrier root to correspond to a blocks-underlay-input layer"
        );

        let listbox = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");

        let mut parent_by_id: std::collections::HashMap<
            fret_core::NodeId,
            Option<fret_core::NodeId>,
        > = std::collections::HashMap::new();
        for n in snap.nodes.iter() {
            parent_by_id.insert(n.id, n.parent);
        }

        let mut root = listbox.id;
        while let Some(parent) = parent_by_id.get(&root).copied().flatten() {
            root = parent;
        }

        assert_eq!(
            root, barrier_root,
            "expected listbox to be rooted under the barrier layer"
        );
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
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn select_scroll_buttons_scroll_without_dismissing() {
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

        let items: Vec<SelectItem> = (0..50)
            .map(|i| SelectItem::new(format!("v{i}"), format!("Item {i}")))
            .collect();

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

        // Third frame: allow the scroll handle to observe content overflow.
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

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let scroll_down = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Scroll down"))
            .expect("scroll down button");
        assert!(
            !snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::Button && n.label.as_deref() == Some("Scroll up")
            }),
            "expected scroll up to be hidden at the top"
        );

        let down_bounds = ui
            .debug_node_bounds(scroll_down.id)
            .expect("scroll down bounds");
        let click = (|| {
            let candidates = [
                (0.5, 0.5),
                (0.25, 0.5),
                (0.75, 0.5),
                (0.5, 0.25),
                (0.5, 0.75),
            ];
            for (fx, fy) in candidates {
                let p = Point::new(
                    Px(down_bounds.origin.x.0 + down_bounds.size.width.0 * fx),
                    Px(down_bounds.origin.y.0 + down_bounds.size.height.0 * fy),
                );
                if let Some(hit) = ui.debug_hit_test(p).hit
                    && ui.debug_node_path(hit).contains(&scroll_down.id)
                {
                    return p;
                }
            }
            panic!("expected scroll down bounds to be hit-testable; bounds={down_bounds:?}");
        })();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click,
                button: MouseButton::Left,
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
            model,
            open.clone(),
            items,
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::Button && n.label.as_deref() == Some("Scroll up")
            }),
            "expected scroll up to appear after scrolling down"
        );
    }
}
