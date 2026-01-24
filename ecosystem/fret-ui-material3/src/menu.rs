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

use crate::foundation::content::MaterialContentDefaults;
use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::indication::{
    IndicationConfig, RippleClip, material_ink_layer_for_pressable,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;

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
}

impl Menu {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn entries(mut self, entries: Vec<MenuEntry>) -> Self {
        self.entries = entries;
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
            let theme = Theme::global(&*cx.app).clone();

            let height = theme
                .metric_by_key("md.comp.menu.list-item.container.height")
                .unwrap_or(Px(48.0));

            let sem = SemanticsProps {
                role: SemanticsRole::Menu,
                label: self.a11y_label.clone(),
                test_id: self.test_id.clone(),
                ..Default::default()
            };

            let mut items: Vec<MenuEntry> = Vec::new();
            items.extend(self.entries.into_iter());

            let mut flat_items: Vec<MenuItem> = Vec::new();
            let mut disabled: Vec<bool> = Vec::new();
            for it in items.iter() {
                match it {
                    MenuEntry::Item(item) => {
                        flat_items.push(item.clone());
                        disabled.push(item.disabled);
                    }
                    MenuEntry::Separator => {}
                }
            }

            let first_enabled_idx = disabled.iter().position(|&d| !d).unwrap_or(0);
            let disabled: Arc<[bool]> = Arc::from(disabled);
            let count = flat_items.len();

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

            let container_bg = theme
                .color_by_key("md.comp.menu.container.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface-container"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container"));
            let elevation = theme
                .metric_by_key("md.comp.menu.container.elevation")
                .unwrap_or(Px(0.0));
            let shadow_color = theme
                .color_by_key("md.comp.menu.container.shadow-color")
                .or_else(|| theme.color_by_key("md.sys.color.shadow"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.shadow"));
            let r = theme
                .metric_by_key("md.comp.menu.container.shape")
                .or_else(|| theme.metric_by_key("md.sys.shape.corner.extra-small"))
                .unwrap_or(Px(4.0));
            let corner = Corners::all(r);
            let shadow =
                shadow_for_elevation_with_color(&theme, elevation, Some(shadow_color), corner);

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
                                Arc::from(
                                    flat_items
                                        .iter()
                                        .map(|it| {
                                            it.a11y_label
                                                .clone()
                                                .unwrap_or_else(|| it.label.clone())
                                        })
                                        .collect::<Vec<_>>(),
                                ),
                                30,
                            );

                            let mut out: Vec<AnyElement> = Vec::with_capacity(items.len());
                            let mut item_idx = 0usize;
                            for entry in items.iter() {
                                match entry {
                                    MenuEntry::Separator => {
                                        out.push(menu_separator(cx, &theme));
                                    }
                                    MenuEntry::Item(it) => {
                                        let tab_stop = item_idx == first_enabled_idx;
                                        out.push(material_menu_item(
                                            cx,
                                            &theme,
                                            it.clone(),
                                            height,
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

fn menu_separator<H: UiHost>(cx: &mut ElementContext<'_, H>, theme: &Theme) -> AnyElement {
    let h = theme
        .metric_by_key("md.comp.menu.divider.height")
        .unwrap_or(Px(1.0));
    let c = theme
        .color_by_key("md.comp.menu.divider.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-variant"));

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
    theme: &Theme,
    item: MenuItem,
    height: Px,
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
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(height);
                l.overflow = Overflow::Visible;
                enforce_minimum_interactive_size(&mut l, theme);
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

                let (label_color, state_layer_color, state_layer_target) =
                    menu_item_outcomes(theme, enabled, is_pressed, is_hovered, is_focused);

                let state_duration_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100);
                let easing = theme
                    .easing_by_key("md.sys.motion.easing.standard")
                    .unwrap_or(fret_ui::theme::CubicBezier {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 1.0,
                        y2: 1.0,
                    });

                let ripple_expand_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short4")
                    .unwrap_or(200);
                let ripple_fade_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100);

                let ripple_base_opacity = theme
                    .number_by_key("md.comp.menu.list-item.pressed.state-layer.opacity")
                    .unwrap_or(0.1);
                let config = IndicationConfig {
                    state_duration_ms,
                    ripple_expand_ms,
                    ripple_fade_ms,
                    ripple_radius: None,
                    easing,
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

                let label_el = menu_item_label(cx, theme, &item.label, label_color);

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
    theme: &Theme,
    text: &Arc<str>,
    color: Color,
) -> AnyElement {
    let style = theme
        .text_style_by_key("md.sys.typescale.label-large")
        .unwrap_or_else(|| TextStyle::default());

    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn menu_item_outcomes(
    theme: &Theme,
    enabled: bool,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> (Color, Color, f32) {
    let (label_key, state_layer_key, opacity_key) = if pressed {
        (
            "md.comp.menu.list-item.pressed.label-text.color",
            "md.comp.menu.list-item.pressed.state-layer.color",
            "md.comp.menu.list-item.pressed.state-layer.opacity",
        )
    } else if focused {
        (
            "md.comp.menu.list-item.focus.label-text.color",
            "md.comp.menu.list-item.focus.state-layer.color",
            "md.comp.menu.list-item.focus.state-layer.opacity",
        )
    } else if hovered {
        (
            "md.comp.menu.list-item.hover.label-text.color",
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        )
    } else {
        (
            "md.comp.menu.list-item.label-text.color",
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        )
    };

    let defaults = MaterialContentDefaults::on_surface(theme);
    let mut label = theme
        .color_by_key(label_key)
        .unwrap_or(defaults.content_color);
    let state_layer = theme
        .color_by_key(state_layer_key)
        .unwrap_or(defaults.content_color);
    let mut opacity = theme.number_by_key(opacity_key).unwrap_or(0.0);

    if !enabled {
        let label_opacity = theme
            .number_by_key("md.comp.menu.list-item.disabled.label-text.opacity")
            .unwrap_or(defaults.disabled_opacity);
        label.a = (label.a * label_opacity).clamp(0.0, 1.0);
        opacity = 0.0;
    }

    (label, state_layer, opacity)
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
