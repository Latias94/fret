//! Material 3 menu (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven container + list-item colors/sizing via `md.comp.menu.*` (subset).
//! - Roving focus + APG-style up/down navigation + optional typeahead.
//! - State layer + bounded ripple on items.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::elements::GlobalElementId;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, merge_override_slot,
    resolve_override_slot_with,
};

use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::surface::material_surface_style;
use crate::tokens::menu as menu_tokens;

#[derive(Debug, Clone, Default)]
pub struct MenuStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub container_corner_radii: OverrideSlot<Corners>,
    pub container_elevation: OverrideSlot<Px>,
    pub item_label_color: OverrideSlot<ColorRef>,
    pub item_state_layer_color: OverrideSlot<ColorRef>,
    pub item_label_text_style: OverrideSlot<TextStyle>,
}

impl MenuStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn container_corner_radii(mut self, corners: WidgetStateProperty<Option<Corners>>) -> Self {
        self.container_corner_radii = Some(corners);
        self
    }

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
        self
    }

    pub fn item_label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.item_label_color = Some(color);
        self
    }

    pub fn item_state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.item_state_layer_color = Some(color);
        self
    }

    pub fn item_label_text_style(mut self, style: WidgetStateProperty<Option<TextStyle>>) -> Self {
        self.item_label_text_style = Some(style);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        Self {
            container_background: merge_override_slot(
                self.container_background,
                other.container_background,
            ),
            container_corner_radii: merge_override_slot(
                self.container_corner_radii,
                other.container_corner_radii,
            ),
            container_elevation: merge_override_slot(
                self.container_elevation,
                other.container_elevation,
            ),
            item_label_color: merge_override_slot(self.item_label_color, other.item_label_color),
            item_state_layer_color: merge_override_slot(
                self.item_state_layer_color,
                other.item_state_layer_color,
            ),
            item_label_text_style: merge_override_slot(
                self.item_label_text_style,
                other.item_label_text_style,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

#[derive(Clone)]
pub struct MenuItem {
    label: Arc<str>,
    pub(crate) disabled: bool,
    pub(crate) on_select: Option<OnActivate>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("label", &self.label)
            .field("disabled", &self.disabled)
            .field("on_select", &self.on_select.is_some())
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl MenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label,
            disabled: false,
            on_select: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, on_select: OnActivate) -> Self {
        self.on_select = Some(on_select);
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
pub struct Menu {
    entries: Vec<MenuEntry>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: MenuStyle,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            a11y_label: None,
            test_id: None,
            style: MenuStyle::default(),
        }
    }

    pub fn entries(mut self, entries: Vec<MenuEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = self.style.merged(style);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_initial_focus_id(cx, Rc::new(std::cell::Cell::new(None)))
    }

    pub(crate) fn into_element_with_initial_focus_id<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let Menu {
                entries,
                a11y_label,
                test_id,
                style,
            } = self;
            let (height, container_bg, shadow, corner) = {
                let theme = Theme::global(&*cx.app);
                let height = menu_tokens::list_item_height(theme);

                let container_bg = resolve_override_slot_with(
                    style.container_background.as_ref(),
                    WidgetStates::empty(),
                    |color| color.resolve(theme),
                    || menu_tokens::container_background(theme),
                );
                let elevation = resolve_override_slot_with(
                    style.container_elevation.as_ref(),
                    WidgetStates::empty(),
                    |v| *v,
                    || menu_tokens::container_elevation(theme),
                );
                let shadow_color = menu_tokens::container_shadow_color(theme);
                let corner = resolve_override_slot_with(
                    style.container_corner_radii.as_ref(),
                    WidgetStates::empty(),
                    |v| *v,
                    || menu_tokens::container_shape(theme),
                );
                let surface = material_surface_style(
                    theme,
                    container_bg,
                    elevation,
                    Some(shadow_color),
                    corner,
                );
                (height, surface.background, surface.shadow, corner)
            };

            let sem = SemanticsProps {
                role: SemanticsRole::Menu,
                label: a11y_label,
                test_id,
                ..Default::default()
            };

            let mut items: Vec<MenuEntry> = Vec::new();
            items.extend(entries.into_iter());

            let mut disabled: Vec<bool> = Vec::new();
            for it in items.iter() {
                match it {
                    MenuEntry::Item(item) => {
                        disabled.push(item.disabled);
                    }
                    MenuEntry::Separator => {}
                }
            }

            let first_enabled_idx = disabled.iter().position(|&d| !d).unwrap_or(0);
            let disabled: Arc<[bool]> = Arc::from(disabled);
            let count = disabled.len();
            let typeahead_items: Arc<[Arc<str>]> = Arc::from(
                items
                    .iter()
                    .filter_map(|it| match it {
                        MenuEntry::Item(item) => Some(
                            item.a11y_label
                                .clone()
                                .unwrap_or_else(|| item.label.clone()),
                        ),
                        MenuEntry::Separator => None,
                    })
                    .collect::<Vec<_>>(),
            );

            let mut roving = RovingFlexProps::default();
            roving.flex.direction = Axis::Vertical;
            roving.flex.gap = Px(0.0);
            roving.flex.align = CrossAlign::Stretch;
            roving.flex.justify = MainAlign::Start;
            roving.roving = fret_ui::element::RovingFocusProps {
                enabled: true,
                wrap: true,
                disabled: disabled.clone(),
            };
            let style: Arc<MenuStyle> = Arc::new(style);

            cx.semantics(sem, move |cx| {
                vec![cx.container(
                    ContainerProps {
                        background: Some(container_bg),
                        shadow,
                        corner_radii: corner,
                        layout: {
                            let mut l = fret_ui::element::LayoutStyle::default();
                            l.size.width = Length::Fill;
                            l.overflow = Overflow::Clip;
                            l
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.roving_flex(roving, move |cx| {
                            cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                                use fret_ui::action::RovingNavigateResult;

                                let is_disabled = |idx: usize| -> bool {
                                    it.disabled.get(idx).copied().unwrap_or(false)
                                };

                                let forward = match it.key {
                                    KeyCode::ArrowDown => Some(true),
                                    KeyCode::ArrowUp => Some(false),
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

                            // Prefix typeahead (best-effort): matches `RadioGroup` semantics in this crate.
                            roving_typeahead_prefix_arc_str_always_wrap(
                                cx,
                                typeahead_items.clone(),
                                30,
                            );

                            let mut out: Vec<AnyElement> = Vec::with_capacity(items.len());
                            let mut item_idx = 0usize;
                            for entry in items.iter() {
                                match entry {
                                    MenuEntry::Separator => {
                                        out.push(menu_separator(cx));
                                    }
                                    MenuEntry::Item(it) => {
                                        let tab_stop = item_idx == first_enabled_idx;
                                        out.push(material_menu_item(
                                            cx,
                                            it.clone(),
                                            height,
                                            style.clone(),
                                            tab_stop,
                                            item_idx,
                                            count,
                                            initial_focus_id_out.clone(),
                                        ));
                                        item_idx += 1;
                                    }
                                }
                            }
                            out
                        })]
                    },
                )]
            })
        })
    }
}

fn menu_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (h, c) = {
        let theme = Theme::global(&*cx.app);
        (
            menu_tokens::divider_height(theme),
            menu_tokens::divider_color(theme),
        )
    };

    let mut props = ContainerProps::default();
    props.background = Some(c);
    props.layout.size.height = Length::Px(h);
    props.layout.size.width = Length::Fill;
    props.layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
    props.layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
    cx.container(props, |_cx| vec![])
}

fn material_menu_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item: MenuItem,
    height: Px,
    style: Arc<MenuStyle>,
    tab_stop: bool,
    idx: usize,
    set_size: usize,
    initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !item.disabled;

        if enabled && tab_stop && initial_focus_id_out.get().is_none() {
            initial_focus_id_out.set(Some(pressable_id));
        }

        let a11y = PressableA11y {
            role: Some(SemanticsRole::MenuItem),
            label: item.a11y_label.clone().or_else(|| Some(item.label.clone())),
            test_id: item.test_id.clone(),
            pos_in_set: Some((idx + 1) as u32),
            set_size: Some(set_size as u32),
            ..Default::default()
        };

        if let Some(handler) = item.on_select.clone() {
            cx.pressable_on_activate(handler);
        }

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
            key_activation: Default::default(),
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(height);
                l.overflow = Overflow::Visible;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut l, theme);
                }
                l
            },
            focus_ring: None,
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

                let interaction = if is_pressed {
                    menu_tokens::MenuItemInteraction::Pressed
                } else if is_focused {
                    menu_tokens::MenuItemInteraction::Focused
                } else if is_hovered {
                    menu_tokens::MenuItemInteraction::Hovered
                } else {
                    menu_tokens::MenuItemInteraction::Default
                };

                let states = WidgetStates::from_pressable(cx, st, enabled);
                let (
                    label_color,
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    label_style,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let (token_label_color, token_state_layer_color, state_layer_target) =
                        menu_tokens::item_outcomes(theme, enabled, interaction);
                    let label_color = resolve_override_slot_with(
                        style.item_label_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || token_label_color,
                    );
                    let state_layer_color = resolve_override_slot_with(
                        style.item_state_layer_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || token_state_layer_color,
                    );

                    let ripple_base_opacity = menu_tokens::pressed_state_layer_opacity(theme);
                    let config = material_pressable_indication_config(theme, None);

                    let default_label_style = theme
                        .text_style_by_key("md.sys.typescale.label-large")
                        .unwrap_or_else(|| TextStyle::default());
                    let label_style = resolve_override_slot_with(
                        style.item_label_text_style.as_ref(),
                        states,
                        |s| s.clone(),
                        || default_label_style,
                    );

                    (
                        label_color,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        label_style,
                    )
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    Corners::all(Px(0.0)),
                    RippleClip::Bounded,
                    state_layer_color,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );
                let label_el = menu_item_label(cx, &item.label, label_style, label_color);

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Start;
                row.align = CrossAlign::Center;
                row.padding = Edges {
                    left: Px(12.0),
                    right: Px(12.0),
                    top: Px(0.0),
                    bottom: Px(0.0),
                };

                vec![cx.flex(row, move |_cx| vec![overlay, label_el])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn menu_item_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &Arc<str>,
    style: TextStyle,
    color: Color,
) -> AnyElement {
    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
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

    let buffer: Rc<RefCell<TypeaheadBuffer>> =
        Rc::new(RefCell::new(TypeaheadBuffer::new(timeout_ticks)));
    let handler: OnRovingTypeahead = Arc::new(
        move |_host: &mut dyn UiActionHost, _cx: ActionCx, it: RovingTypeaheadCx| {
            let tick = it.tick;
            let ch = it.input;

            let mut buf = buffer.borrow_mut();
            buf.push_char(ch, tick);
            let query = buf.active_query(tick)?;

            let current = it.current.unwrap_or(0);
            let mut matches: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(idx, label)| {
                    let label = label.to_lowercase();
                    label.starts_with(query).then_some(idx)
                })
                .collect();
            if matches.is_empty() {
                return None;
            }

            // If query is a single character, skip the current match to allow cycling.
            if query.chars().count() == 1 {
                matches.retain(|&idx| idx != current);
                if matches.is_empty() {
                    return None;
                }
            }

            // Always wrap: prefer the next match after current, otherwise the first.
            let next = matches
                .iter()
                .copied()
                .find(|&idx| idx > current)
                .or_else(|| matches.into_iter().next());
            next
        },
    );

    cx.roving_add_on_typeahead(handler);
}
