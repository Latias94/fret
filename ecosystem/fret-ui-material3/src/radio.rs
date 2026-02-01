//! Material 3 radio button (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.radio-button.*`.
//! - State layer (hover/pressed/focus) + unbounded ripple using `fret_ui::paint`.
//! - Inner dot grow animation on selection (best-effort).

use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use fret_core::{
    Color, Corners, DrawOrder, Edges, KeyCode, Point, Px, Rect, SceneOp, SemanticsRole, Size,
};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, Length, Overflow, PointerRegionProps, PressableA11y,
    PressableProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::pixel_snap;
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::interaction::state_layer::StateLayerAnimator;
use crate::tokens::radio as radio_tokens;
use crate::tokens::radio::RadioSizeTokens;

/// Matches Material (and WAI-ARIA APG) `RadioGroup` orientation outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioGroupOrientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct RadioGroupItem {
    value: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl RadioGroupItem {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroup {
    model: Model<Option<Arc<str>>>,
    items: Arc<[RadioGroupItem]>,
    disabled: bool,
    orientation: RadioGroupOrientation,
    gap: Px,
    typeahead_timeout_ticks: u64,
    loop_navigation: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: RadioStyle,
}

impl RadioGroup {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Arc::from([]),
            disabled: false,
            orientation: RadioGroupOrientation::default(),
            gap: Px(0.0),
            typeahead_timeout_ticks: 60,
            loop_navigation: true,
            a11y_label: None,
            test_id: None,
            style: RadioStyle::default(),
        }
    }

    pub fn items(mut self, items: impl Into<Arc<[RadioGroupItem]>>) -> Self {
        self.items = items.into();
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = gap;
        self
    }

    pub fn style(mut self, style: RadioStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    /// Configure the prefix-buffer typeahead timeout in `TickId` units (default: `60`).
    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks.max(1);
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let model = self.model.clone();
            let items = self.items.clone();
            let disabled_group = self.disabled;
            let label = self.a11y_label.clone();
            let test_id = self.test_id.clone();
            let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
            let style = self.style.clone();

            let values: Arc<[Arc<str>]> =
                Arc::from(items.iter().map(|it| it.value.clone()).collect::<Vec<_>>());
            let disabled: Arc<[bool]> = Arc::from(
                items
                    .iter()
                    .map(|it| disabled_group || it.disabled)
                    .collect::<Vec<_>>(),
            );

            let typeahead_labels: Arc<[Arc<str>]> = Arc::from(
                items
                    .iter()
                    .map(|it| it.a11y_label.clone().unwrap_or_else(|| it.value.clone()))
                    .collect::<Vec<_>>(),
            );

            let selected = cx.get_model_cloned(&model, Invalidation::Layout).flatten();
            let tab_stop = selected.as_ref().and_then(|selected| {
                items.iter().position(|it| {
                    !disabled_group && !it.disabled && it.value.as_ref() == selected.as_ref()
                })
            });
            let tab_stop =
                tab_stop.or_else(|| items.iter().position(|it| !disabled_group && !it.disabled));

            let set_size = values.len();
            let sem = fret_ui::element::SemanticsProps {
                role: SemanticsRole::RadioGroup,
                label,
                test_id,
                disabled: disabled_group,
                ..Default::default()
            };

            let mut props = fret_ui::element::RovingFlexProps::default();
            props.flex.direction = match self.orientation {
                RadioGroupOrientation::Vertical => fret_core::Axis::Vertical,
                RadioGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
            };
            props.flex.gap = self.gap;
            props.flex.align = fret_ui::element::CrossAlign::Center;
            props.roving = fret_ui::element::RovingFocusProps {
                enabled: !disabled_group,
                wrap: self.loop_navigation,
                disabled: disabled.clone(),
            };

            cx.semantics(sem, move |cx| {
                vec![cx.roving_flex(props, move |cx| {
                    let values_for_roving = values.clone();
                    let model_for_roving = model.clone();
                    cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                        use fret_ui::action::RovingNavigateResult;

                        let is_disabled =
                            |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                        let forward = match (it.axis, it.key) {
                            (fret_core::Axis::Vertical, KeyCode::ArrowDown) => Some(true),
                            (fret_core::Axis::Vertical, KeyCode::ArrowUp) => Some(false),
                            (fret_core::Axis::Horizontal, KeyCode::ArrowRight) => Some(true),
                            (fret_core::Axis::Horizontal, KeyCode::ArrowLeft) => Some(false),
                            _ => None,
                        };

                        if it.key == KeyCode::Home {
                            let target = (0..it.len).find(|&i| !is_disabled(i));
                            return RovingNavigateResult::Handled { target };
                        }
                        if it.key == KeyCode::End {
                            let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                            return RovingNavigateResult::Handled { target };
                        }

                        let Some(forward) = forward else {
                            return RovingNavigateResult::NotHandled;
                        };

                        let current = it
                            .current
                            .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                        let Some(current) = current else {
                            return RovingNavigateResult::Handled { target: None };
                        };

                        let len = it.len;
                        let mut target: Option<usize> = None;
                        if it.wrap {
                            for step in 1..=len {
                                let idx = if forward {
                                    (current + step) % len
                                } else {
                                    (current + len - (step % len)) % len
                                };
                                if !is_disabled(idx) {
                                    target = Some(idx);
                                    break;
                                }
                            }
                        } else if forward {
                            target = ((current + 1)..len).find(|&i| !is_disabled(i));
                        } else if current > 0 {
                            target = (0..current).rev().find(|&i| !is_disabled(i));
                        }

                        RovingNavigateResult::Handled { target }
                    }));

                    cx.roving_on_active_change(Arc::new(move |host, action_cx, idx| {
                        let Some(value) = values_for_roving.get(idx).cloned() else {
                            return;
                        };
                        let already_selected = host
                            .models_mut()
                            .read(&model_for_roving, |v| {
                                v.as_ref()
                                    .is_some_and(|current| current.as_ref() == value.as_ref())
                            })
                            .ok()
                            .unwrap_or(false);
                        if already_selected {
                            return;
                        }
                        let next = Some(value);
                        let _ = host.update_model(&model_for_roving, |v| *v = next);
                        host.request_redraw(action_cx.window);
                    }));

                    roving_typeahead_prefix_arc_str_always_wrap(
                        cx,
                        typeahead_labels.clone(),
                        typeahead_timeout_ticks,
                    );

                    items
                        .iter()
                        .enumerate()
                        .map(|(idx, it)| {
                            let mut radio = Radio::new_value(it.value.clone(), model.clone())
                                .style(style.clone())
                                .disabled(disabled_group || it.disabled);
                            if let Some(label) = it.a11y_label.as_ref() {
                                radio = radio.a11y_label(label.clone());
                            }
                            if let Some(test_id) = it.test_id.as_ref() {
                                radio = radio.test_id(test_id.clone());
                            }
                            radio = radio
                                .roving_tab_stop(tab_stop.is_some_and(|t| t == idx))
                                .collection_position(idx, set_size);
                            radio.into_element(cx)
                        })
                        .collect::<Vec<_>>()
                })]
            })
        })
    }
}

fn roving_typeahead_prefix_arc_str_always_wrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
) {
    use fret_ui::action::{ActionCx, OnRovingTypeahead, RovingTypeaheadCx, UiActionHost};

    #[derive(Debug, Default)]
    struct TypeaheadBuffer {
        timeout_ticks: u64,
        last_tick: Option<u64>,
        query: String,
    }

    impl TypeaheadBuffer {
        fn new(timeout_ticks: u64) -> Self {
            Self {
                timeout_ticks,
                last_tick: None,
                query: String::new(),
            }
        }

        fn push_char(&mut self, ch: char, tick: u64) {
            if ch.is_whitespace() {
                return;
            }
            let expired = self
                .last_tick
                .is_some_and(|last| tick.saturating_sub(last) > self.timeout_ticks);
            if expired {
                self.query.clear();
            }
            self.last_tick = Some(tick);
            self.query.extend(ch.to_lowercase());
        }

        fn active_query(&mut self, tick: u64) -> Option<&str> {
            let expired = self
                .last_tick
                .is_some_and(|last| tick.saturating_sub(last) > self.timeout_ticks);
            if expired {
                self.query.clear();
                self.last_tick = None;
                return None;
            }
            if self.query.is_empty() {
                None
            } else {
                Some(self.query.as_str())
            }
        }
    }

    fn match_prefix_arc_str(
        labels: &[Arc<str>],
        disabled: &[bool],
        query: &str,
        current: Option<usize>,
    ) -> Option<usize> {
        if labels.is_empty() {
            return None;
        }

        let query = query.trim();
        if query.is_empty() {
            return None;
        }

        fn normalize_repeated_search(query: &str) -> String {
            let mut it = query.chars();
            let Some(first) = it.next() else {
                return query.to_string();
            };
            let mut count = 1usize;
            for c in it {
                count += 1;
                if c != first {
                    return query.to_string();
                }
            }
            if count <= 1 {
                query.to_string()
            } else {
                first.to_string()
            }
        }

        let query = normalize_repeated_search(query);
        let exclude_current_match = query.chars().count() == 1 && current.is_some();

        let is_disabled = |idx: usize| disabled.get(idx).copied().unwrap_or(false);
        let matches = |idx: usize| -> bool {
            if is_disabled(idx) {
                return false;
            }
            let Some(label) = labels.get(idx) else {
                return false;
            };
            label.trim_start().to_ascii_lowercase().starts_with(&query)
        };

        let len = labels.len();
        let start = current.unwrap_or(0);
        let start = if exclude_current_match {
            start.saturating_add(1)
        } else {
            start
        };
        for offset in 0..len {
            let idx = (start + offset) % len;
            if matches(idx) {
                return Some(idx);
            }
        }
        None
    }

    struct TypeaheadState {
        timeout_ticks: u64,
        labels: Rc<RefCell<Arc<[Arc<str>]>>>,
        handler: OnRovingTypeahead,
    }

    fn make_state(labels: Arc<[Arc<str>]>, timeout_ticks: u64) -> TypeaheadState {
        let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> = Rc::new(RefCell::new(labels));
        let buffer: Rc<RefCell<TypeaheadBuffer>> =
            Rc::new(RefCell::new(TypeaheadBuffer::new(timeout_ticks)));

        let labels_read = labels_cell.clone();
        let buffer_read = buffer.clone();

        #[allow(clippy::arc_with_non_send_sync)]
        let handler: OnRovingTypeahead = Arc::new(
            move |_host: &mut dyn UiActionHost, _cx: ActionCx, it: RovingTypeaheadCx| {
                let mut buf = buffer_read.borrow_mut();
                buf.push_char(it.input, it.tick);
                let Some(query) = buf.active_query(it.tick) else {
                    return None;
                };

                let labels = labels_read.borrow();
                match_prefix_arc_str(labels.as_ref(), it.disabled.as_ref(), query, it.current)
            },
        );

        TypeaheadState {
            timeout_ticks,
            labels: labels_cell,
            handler,
        }
    }

    let handler = cx.with_state(
        || make_state(labels.clone(), timeout_ticks),
        |state| {
            if state.timeout_ticks != timeout_ticks {
                *state = make_state(labels.clone(), timeout_ticks);
            }
            *state.labels.borrow_mut() = labels.clone();
            state.handler.clone()
        },
    );

    cx.roving_add_on_typeahead(handler);
}

#[derive(Debug, Clone)]
enum RadioSelectionModel {
    Bool(Model<bool>),
    Group {
        value: Arc<str>,
        selected_value: Model<Option<Arc<str>>>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct RadioStyle {
    pub icon_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl RadioStyle {
    pub fn icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.icon_color.is_some() {
            self.icon_color = other.icon_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Radio {
    selection: RadioSelectionModel,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    focus_policy: RadioFocusPolicy,
    a11y_pos_in_set: Option<u32>,
    a11y_set_size: Option<u32>,
    on_activate: Option<OnActivate>,
    style: RadioStyle,
}

#[derive(Debug, Clone, Copy)]
enum RadioFocusPolicy {
    Default,
    Roving { tab_stop: bool },
}

impl std::fmt::Debug for Radio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Radio")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .finish()
    }
}

impl Radio {
    /// A standalone radio bound to a `Model<bool>`.
    ///
    /// Note: activation sets the model to `true` (does not toggle off).
    pub fn new(selected: Model<bool>) -> Self {
        Self {
            selection: RadioSelectionModel::Bool(selected),
            disabled: false,
            a11y_label: None,
            test_id: None,
            focus_policy: RadioFocusPolicy::Default,
            a11y_pos_in_set: None,
            a11y_set_size: None,
            on_activate: None,
            style: RadioStyle::default(),
        }
    }

    /// A radio item bound to a shared group-value model.
    pub fn new_value(value: impl Into<Arc<str>>, group_value: Model<Option<Arc<str>>>) -> Self {
        Self {
            selection: RadioSelectionModel::Group {
                value: value.into(),
                selected_value: group_value,
            },
            disabled: false,
            a11y_label: None,
            test_id: None,
            focus_policy: RadioFocusPolicy::Default,
            a11y_pos_in_set: None,
            a11y_set_size: None,
            on_activate: None,
            style: RadioStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Enable roving-focus-friendly tab stop behavior.
    ///
    /// When enabled, only the current tab stop (or the currently focused item) is included in the
    /// default focus traversal order.
    pub fn roving_tab_stop(mut self, tab_stop: bool) -> Self {
        self.focus_policy = RadioFocusPolicy::Roving { tab_stop };
        self
    }

    /// Populate collection metadata for accessibility (`pos_in_set`, `set_size`).
    ///
    /// `index` is 0-based; it is mapped to a 1-based `pos_in_set` value.
    pub fn collection_position(mut self, index: usize, set_size: usize) -> Self {
        if set_size == 0 || index >= set_size {
            self.a11y_pos_in_set = None;
            self.a11y_set_size = None;
            return self;
        }

        let pos_in_set = index.saturating_add(1);
        self.a11y_pos_in_set = u32::try_from(pos_in_set).ok();
        self.a11y_set_size = u32::try_from(set_size).ok();

        if let (Some(pos), Some(size)) = (self.a11y_pos_in_set, self.a11y_set_size)
            && pos > size
        {
            self.a11y_pos_in_set = None;
            self.a11y_set_size = None;
        }

        self
    }

    /// Called after the radio updates its selection model.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: RadioStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let size = radio_tokens::size_tokens(&theme);

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                cx.key_add_on_key_down_for(pressable_id, consume_enter_key_handler());

                let selection_for_activate = self.selection.clone();
                let enabled_for_toggle = enabled;
                let user_activate = self.on_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    if enabled_for_toggle {
                        match &selection_for_activate {
                            RadioSelectionModel::Bool(m) => {
                                let already_selected =
                                    host.models_mut().read(m, |v| *v).ok().unwrap_or(false);
                                if !already_selected {
                                    let _ = host.update_model(m, |v| *v = true);
                                }
                            }
                            RadioSelectionModel::Group {
                                value,
                                selected_value,
                            } => {
                                let value = value.clone();
                                let already_selected = host
                                    .models_mut()
                                    .read(selected_value, |v| {
                                        v.as_ref().is_some_and(|current| {
                                            current.as_ref() == value.as_ref()
                                        })
                                    })
                                    .ok()
                                    .unwrap_or(false);
                                if !already_selected {
                                    let _ = host.update_model(selected_value, |v| *v = Some(value));
                                }
                            }
                        }
                        host.request_redraw(action_cx.window);
                    }
                    if let Some(h) = user_activate.as_ref() {
                        h(host, action_cx, reason);
                    }
                }));

                let checked = match &self.selection {
                    RadioSelectionModel::Bool(m) => cx
                        .get_model_copied(m, Invalidation::Layout)
                        .unwrap_or(false),
                    RadioSelectionModel::Group {
                        value,
                        selected_value,
                    } => cx
                        .get_model_cloned(selected_value, Invalidation::Layout)
                        .flatten()
                        .is_some_and(|v| v.as_ref() == value.as_ref()),
                };

                let corner_radii = theme
                    .corners_by_key("md.sys.shape.corner.full")
                    .unwrap_or_else(|| Corners::all(Px(9999.0)));
                let focusable = match self.focus_policy {
                    RadioFocusPolicy::Default => enabled,
                    RadioFocusPolicy::Roving { tab_stop } => enabled && (tab_stop || st.focused),
                };
                let pressable_props = PressableProps {
                    enabled,
                    focusable,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::RadioButton),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(checked),
                        pos_in_set: self.a11y_pos_in_set,
                        set_size: self.a11y_set_size,
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        enforce_minimum_interactive_size(&mut l, &theme);
                        l
                    },
                    focus_ring: Some(material_focus_ring_for_component(
                        &theme,
                        "md.comp.radio-button",
                        corner_radii,
                    )),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    cx.pointer_region(props, |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let now_frame = cx.frame_id.0;
                        let focus_visible =
                            fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                        let is_pressed = enabled && st.pressed;
                        let is_hovered = enabled && st.hovered;
                        let is_focused = enabled && st.focused && focus_visible;
                        let tokens_interaction =
                            match pressable_interaction(is_pressed, is_hovered, is_focused) {
                                Some(PressableInteraction::Pressed) => {
                                    radio_tokens::RadioInteraction::Pressed
                                }
                                Some(PressableInteraction::Focused) => {
                                    radio_tokens::RadioInteraction::Focused
                                }
                                Some(PressableInteraction::Hovered) => {
                                    radio_tokens::RadioInteraction::Hovered
                                }
                                None => radio_tokens::RadioInteraction::None,
                            };

                        let checked = match &self.selection {
                            RadioSelectionModel::Bool(m) => {
                                cx.get_model_copied(m, Invalidation::Paint).unwrap_or(false)
                            }
                            RadioSelectionModel::Group {
                                value,
                                selected_value,
                            } => cx
                                .get_model_cloned(selected_value, Invalidation::Paint)
                                .flatten()
                                .is_some_and(|v| v.as_ref() == value.as_ref()),
                        };

                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        if checked {
                            states |= WidgetStates::SELECTED;
                        }

                        let state_layer_target = radio_tokens::state_layer_target_opacity(
                            &theme,
                            checked,
                            enabled,
                            tokens_interaction,
                        );
                        let state_layer_color =
                            radio_tokens::state_layer_color(&theme, checked, tokens_interaction);
                        let state_layer_color = resolve_override_slot_with(
                            self.style.state_layer_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || state_layer_color,
                        );
                        let indication_config = material_pressable_indication_config(
                            &theme,
                            Some(Px(size.state_layer.0 * 0.5)),
                        );

                        let dot_duration_ms = theme
                            .duration_ms_by_key("md.sys.motion.duration.medium2")
                            .unwrap_or(300);
                        let dot_easing = theme
                            .easing_by_key("md.sys.motion.easing.emphasized.decelerate")
                            .unwrap_or(indication_config.easing);

                        #[derive(Default)]
                        struct RadioDotRuntime {
                            dot_target: f32,
                            dot: StateLayerAnimator,
                        }

                        let (dot_scale, dot_active) =
                            cx.with_state_for(pressable_id, RadioDotRuntime::default, |rt| {
                                let desired_dot = if checked { 1.0 } else { 0.0 };
                                if (desired_dot - rt.dot_target).abs() > 1e-6 {
                                    rt.dot_target = desired_dot;
                                    rt.dot.set_target(
                                        now_frame,
                                        desired_dot,
                                        dot_duration_ms,
                                        dot_easing,
                                    );
                                }
                                rt.dot.advance(now_frame);
                                (rt.dot.value(), rt.dot.is_active())
                            });

                        let ripple_base_opacity =
                            radio_tokens::pressed_state_layer_opacity(&theme, checked);
                        let overlay = material_ink_layer_for_pressable(
                            cx,
                            pressable_id,
                            now_frame,
                            corner_radii,
                            RippleClip::Bounded,
                            state_layer_color,
                            is_pressed,
                            state_layer_target,
                            ripple_base_opacity,
                            indication_config,
                            dot_active,
                        );

                        let icon_color =
                            radio_tokens::icon_color(&theme, checked, enabled, tokens_interaction);
                        let icon_color = resolve_override_slot_with(
                            self.style.icon_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || icon_color,
                        );
                        let icon = radio_icon(cx, &theme, size, checked, icon_color, dot_scale);

                        let chrome = material_radio_chrome(cx, size, vec![overlay, icon]);
                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn material_radio_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: RadioSizeTokens,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.corner_radii = Corners::all(Px(size.state_layer.0 * 0.5));
    props.layout.size.width = Length::Px(size.state_layer);
    props.layout.size.height = Length::Px(size.state_layer);
    cx.container(props, move |_cx| children)
}

fn radio_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    size: RadioSizeTokens,
    checked: bool,
    color: Color,
    dot_scale: f32,
) -> AnyElement {
    // Note: this intentionally uses a Canvas-backed icon to avoid relying on absolute child
    // positioning semantics for the inner dot.
    let outline_width = Px(2.0);
    let dot_max = Px(10.0);
    let dot_size = Px(dot_max.0 * dot_scale.clamp(0.0, 1.0));

    let mut props = CanvasProps::default();
    props.layout.position = fret_ui::element::PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0));
    props.layout.inset.right = Some(Px(0.0));
    props.layout.inset.bottom = Some(Px(0.0));
    props.layout.inset.left = Some(Px(0.0));

    cx.canvas(props, move |p| {
        let scale_factor = p.scale_factor();
        let bounds = p.bounds();

        let icon_left = Px(bounds.origin.x.0 + (bounds.size.width.0 - size.icon.0) * 0.5);
        let icon_top = Px(bounds.origin.y.0 + (bounds.size.height.0 - size.icon.0) * 0.5);
        let icon_rect = pixel_snap::snap_rect_edges_round(
            Rect::new(
                Point::new(icon_left, icon_top),
                Size::new(size.icon, size.icon),
            ),
            scale_factor,
        );
        let icon_radius = Px(size.icon.0 * 0.5);

        p.scene().push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: icon_rect,
            background: Color::TRANSPARENT,
            border: Edges::all(outline_width),
            border_color: color,
            corner_radii: Corners::all(icon_radius),
        });

        if checked || dot_size.0 > 0.1 {
            let dot_left = Px(icon_rect.origin.x.0 + (icon_rect.size.width.0 - dot_size.0) * 0.5);
            let dot_top = Px(icon_rect.origin.y.0 + (icon_rect.size.height.0 - dot_size.0) * 0.5);
            let dot_left = pixel_snap::snap_px_round(dot_left, scale_factor);
            let dot_top = pixel_snap::snap_px_round(dot_top, scale_factor);
            let dot_rect = Rect::new(Point::new(dot_left, dot_top), Size::new(dot_size, dot_size));
            p.scene().push(SceneOp::Quad {
                order: DrawOrder(1),
                rect: dot_rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(dot_size.0 * 0.5)),
            });
        }
    })
}

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}
