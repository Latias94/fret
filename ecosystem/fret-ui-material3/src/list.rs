//! Material 3 list (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing and colors via `md.comp.list.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SemanticsProps,
    SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};

use crate::foundation::content::MaterialContentDefaults;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    IndicationConfig, RippleClip, advance_indication_for_pressable, material_ink_layer,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::token_resolver::MaterialTokenResolver;

#[derive(Debug, Clone)]
pub struct ListItem {
    value: Arc<str>,
    label: Arc<str>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl ListItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
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
}

#[derive(Debug, Clone)]
pub struct List {
    model: Model<Arc<str>>,
    items: Vec<ListItem>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
    loop_navigation: bool,
}

impl List {
    pub fn new(model: Model<Arc<str>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            a11y_label: None,
            test_id: None,
            disabled: false,
            loop_navigation: true,
        }
    }

    pub fn items(mut self, items: Vec<ListItem>) -> Self {
        self.items = items;
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let List {
            model,
            items,
            a11y_label,
            test_id,
            disabled,
            loop_navigation,
        } = self;

        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let sem = SemanticsProps {
                role: SemanticsRole::List,
                label: a11y_label.clone(),
                test_id: test_id.clone(),
                ..Default::default()
            };

            let disabled_items: Arc<[bool]> =
                Arc::from(items.iter().map(|it| it.disabled).collect::<Vec<_>>());
            let values_for_roving: Arc<[Arc<str>]> =
                Arc::from(items.iter().map(|it| it.value.clone()).collect::<Vec<_>>());
            let count = items.len();

            let selected_idx =
                cx.get_model_cloned(&model, Invalidation::Layout)
                    .and_then(|value| {
                        items
                            .iter()
                            .position(|it| it.value.as_ref() == value.as_ref())
                    });

            let tab_stop = selected_idx.or_else(|| disabled_items.iter().position(|&d| !d));
            let model_for_roving = model.clone();

            let mut roving = RovingFlexProps::default();
            roving.flex.direction = Axis::Vertical;
            roving.flex.gap = Px(0.0);
            roving.flex.align = CrossAlign::Stretch;
            roving.flex.justify = MainAlign::Start;
            roving.roving = fret_ui::element::RovingFocusProps {
                enabled: true,
                wrap: loop_navigation,
                disabled: disabled_items.clone(),
            };

            cx.semantics(sem, move |cx| {
                vec![cx.container(
                    ContainerProps {
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

                                if it.key == KeyCode::Home {
                                    let target = (0..it.len).find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }
                                if it.key == KeyCode::End {
                                    let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }

                                let forward = match it.key {
                                    KeyCode::ArrowDown => Some(true),
                                    KeyCode::ArrowUp => Some(false),
                                    _ => None,
                                };
                                let Some(forward) = forward else {
                                    return RovingNavigateResult::NotHandled;
                                };

                                let current = it
                                    .current
                                    .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                                let Some(current) = current else {
                                    return RovingNavigateResult::Handled { target: None };
                                };

                                let mut idx = current;
                                for _ in 0..it.len {
                                    idx = if forward {
                                        if idx + 1 < it.len { idx + 1 } else { 0 }
                                    } else if idx > 0 {
                                        idx - 1
                                    } else {
                                        it.len.saturating_sub(1)
                                    };

                                    if !is_disabled(idx) {
                                        return RovingNavigateResult::Handled { target: Some(idx) };
                                    }
                                }

                                RovingNavigateResult::Handled { target: None }
                            }));

                            cx.roving_on_active_change(Arc::new(move |host, action_cx, idx| {
                                let Some(value) = values_for_roving.get(idx).cloned() else {
                                    return;
                                };
                                let already_selected = host
                                    .models_mut()
                                    .read(&model_for_roving, |v| v.as_ref() == value.as_ref())
                                    .ok()
                                    .unwrap_or(false);
                                if already_selected {
                                    return;
                                }
                                let _ = host.update_model(&model_for_roving, |v| *v = value);
                                host.request_redraw(action_cx.window);
                            }));

                            items
                                .iter()
                                .enumerate()
                                .map(|(idx, item)| {
                                    let tab_stop = tab_stop.is_some_and(|t| t == idx);
                                    list_item(
                                        cx,
                                        &theme,
                                        model.clone(),
                                        item,
                                        idx,
                                        count,
                                        tab_stop,
                                        disabled,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })]
                    },
                )]
            })
        })
    }
}

fn list_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    model: Model<Arc<str>>,
    item: &ListItem,
    idx: usize,
    set_size: usize,
    tab_stop: bool,
    disabled_group: bool,
) -> AnyElement {
    let value = item.value.clone();
    let label = item.label.clone();
    let leading_icon = item.leading_icon.clone();
    let trailing_icon = item.trailing_icon.clone();
    let a11y_label = item.a11y_label.clone();
    let test_id = item.test_id.clone();

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !disabled_group && !item.disabled;
        let selected = cx
            .get_model_cloned(&model, Invalidation::Layout)
            .map(|v| v.as_ref() == value.as_ref())
            .unwrap_or(false);

        if enabled {
            let model_for_press = model.clone();
            let value_for_press = value.clone();
            let handler: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let already_selected = host
                    .models_mut()
                    .read(&model_for_press, |v| v.as_ref() == value_for_press.as_ref())
                    .ok()
                    .unwrap_or(false);
                if already_selected {
                    return;
                }
                let _ = host.update_model(&model_for_press, |v| *v = value_for_press.clone());
                host.request_redraw(action_cx.window);
            });
            cx.pressable_on_activate(handler);
        }

        let tokens = MaterialTokenResolver::new(theme);
        let height = theme
            .metric_by_key("md.comp.list.list-item.one-line.container.height")
            .unwrap_or(Px(56.0));

        let corner_radii = theme
            .corners_by_key("md.comp.list.list-item.container.shape")
            .unwrap_or(Corners::all(Px(0.0)));

        let focus_ring = material_focus_ring_for_component(theme, "md.comp.list", corner_radii);

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
            a11y: PressableA11y {
                role: Some(SemanticsRole::ListItem),
                label: a11y_label.clone().or_else(|| Some(label.clone())),
                test_id: test_id.clone(),
                selected,
                pos_in_set: Some((idx + 1) as u32),
                set_size: Some(set_size as u32),
                ..Default::default()
            },
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(height);
                l.overflow = Overflow::Visible;
                enforce_minimum_interactive_size(&mut l, theme);
                l
            },
            focus_ring: Some(focus_ring),
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

                let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                let (label_color, icon_color, state_layer_color, state_layer_target) =
                    list_item_outcomes(theme, selected, enabled, interaction);

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

                let bounds = cx
                    .last_bounds_for_element(cx.root_id())
                    .unwrap_or(cx.bounds);
                let last_down = cx
                    .with_state(fret_ui::element::PointerRegionState::default, |st| {
                        st.last_down
                    });

                let ripple_base_opacity = list_item_ripple_base_opacity(theme, selected);
                let config = IndicationConfig {
                    state_duration_ms,
                    ripple_expand_ms,
                    ripple_fade_ms,
                    ripple_radius: None,
                    easing,
                };
                let indication = advance_indication_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    bounds,
                    last_down,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                );

                let overlay = material_ink_layer(
                    cx,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    indication.state_layer_opacity,
                    indication.ripple_frame,
                    indication.want_frames,
                );

                let selected_bg = if selected {
                    if enabled {
                        Some(tokens.color_comp_or_sys(
                            "md.comp.list.list-item.selected.container.color",
                            "md.sys.color.secondary-container",
                        ))
                    } else {
                        let mut bg = tokens.color_comp_or_sys(
                            "md.comp.list.list-item.selected.disabled.container.color",
                            "md.sys.color.on-surface",
                        );
                        let opacity = theme
                            .number_by_key(
                                "md.comp.list.list-item.selected.disabled.container.opacity",
                            )
                            .unwrap_or(0.38);
                        bg.a = (bg.a * opacity).clamp(0.0, 1.0);
                        Some(bg)
                    }
                } else {
                    None
                };

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Start;
                row.align = CrossAlign::Center;
                row.gap = theme
                    .metric_by_key("md.comp.list.list-item.between-space")
                    .unwrap_or(Px(12.0));
                row.padding = Edges {
                    left: theme
                        .metric_by_key("md.comp.list.list-item.leading-space")
                        .unwrap_or(Px(16.0)),
                    right: theme
                        .metric_by_key("md.comp.list.list-item.trailing-space")
                        .unwrap_or(Px(16.0)),
                    top: theme
                        .metric_by_key("md.comp.list.list-item.top-space")
                        .unwrap_or(Px(10.0)),
                    bottom: theme
                        .metric_by_key("md.comp.list.list-item.bottom-space")
                        .unwrap_or(Px(10.0)),
                };

                let container = cx.container(
                    ContainerProps {
                        background: selected_bg,
                        corner_radii,
                        layout: {
                            let mut l = fret_ui::element::LayoutStyle::default();
                            l.size.width = Length::Fill;
                            l.size.height = Length::Px(height);
                            l.overflow = Overflow::Clip;
                            l
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let leading = leading_icon
                            .as_ref()
                            .map(|id| list_icon(cx, theme, id, icon_color, ListIconSlot::Leading));
                        let trailing = trailing_icon
                            .as_ref()
                            .map(|id| list_icon(cx, theme, id, icon_color, ListIconSlot::Trailing));
                        let label_el = list_item_label(cx, theme, &label, label_color);

                        let content = cx.flex(row, move |_cx| {
                            let mut out = Vec::new();
                            if let Some(leading) = leading {
                                out.push(leading);
                            }
                            out.push(label_el);
                            if let Some(trailing) = trailing {
                                out.push(trailing);
                            }
                            out
                        });

                        vec![overlay, content]
                    },
                );

                vec![container]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interaction {
    None,
    Hovered,
    Focused,
    Pressed,
}

fn interaction_state(pressed: bool, hovered: bool, focused: bool) -> Interaction {
    if pressed {
        Interaction::Pressed
    } else if focused {
        Interaction::Focused
    } else if hovered {
        Interaction::Hovered
    } else {
        Interaction::None
    }
}

fn list_item_outcomes(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    it: Interaction,
) -> (Color, Color, Color, f32) {
    let defaults = MaterialContentDefaults::on_surface(theme);
    let tokens = MaterialTokenResolver::new(theme);

    let (label_key, icon_key, state_layer_key, opacity_key) = match (selected, it) {
        (true, Interaction::Pressed) => (
            "md.comp.list.list-item.selected.pressed.label-text.color",
            "md.comp.list.list-item.selected.pressed.leading-icon.color",
            "md.comp.list.list-item.selected.pressed.state-layer.color",
            "md.comp.list.list-item.selected.pressed.state-layer.opacity",
        ),
        (true, Interaction::Focused) => (
            "md.comp.list.list-item.selected.focus.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.focus.state-layer.color",
            "md.comp.list.list-item.selected.focus.state-layer.opacity",
        ),
        (true, Interaction::Hovered) => (
            "md.comp.list.list-item.selected.hover.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (true, Interaction::None) => (
            "md.comp.list.list-item.selected.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (false, Interaction::Pressed) => (
            "md.comp.list.list-item.pressed.label-text.color",
            "md.comp.list.list-item.pressed.leading-icon.icon.color",
            "md.comp.list.list-item.pressed.state-layer.color",
            "md.comp.list.list-item.pressed.state-layer.opacity",
        ),
        (false, Interaction::Focused) => (
            "md.comp.list.list-item.focus.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.focus.state-layer.color",
            "md.comp.list.list-item.focus.state-layer.opacity",
        ),
        (false, Interaction::Hovered) => (
            "md.comp.list.list-item.hover.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
        (false, Interaction::None) => (
            "md.comp.list.list-item.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
    };

    let mut label = theme
        .color_by_key(label_key)
        .unwrap_or(defaults.content_color);
    let mut icon = theme
        .color_by_key(icon_key)
        .unwrap_or(tokens.color_sys("md.sys.color.on-surface-variant"));
    let state_layer = theme
        .color_by_key(state_layer_key)
        .unwrap_or(defaults.content_color);
    let mut opacity = theme.number_by_key(opacity_key).unwrap_or(0.0);

    if it == Interaction::None {
        opacity = 0.0;
    }

    if !enabled {
        let (
            disabled_label_key,
            disabled_label_opacity_key,
            disabled_icon_key,
            disabled_icon_opacity_key,
        ) = if selected {
            (
                "md.comp.list.list-item.selected.disabled.label-text.color",
                "md.comp.list.list-item.selected.disabled.label-text.opacity",
                "md.comp.list.list-item.selected.disabled.leading-icon.color",
                "md.comp.list.list-item.selected.disabled.leading-icon.opacity",
            )
        } else {
            (
                "md.comp.list.list-item.disabled.label-text.color",
                "md.comp.list.list-item.disabled.label-text.opacity",
                "md.comp.list.list-item.disabled.leading-icon.color",
                "md.comp.list.list-item.disabled.leading-icon.opacity",
            )
        };

        label = theme
            .color_by_key(disabled_label_key)
            .unwrap_or(defaults.content_color);
        icon = theme
            .color_by_key(disabled_icon_key)
            .unwrap_or(tokens.color_sys("md.sys.color.on-surface-variant"));

        let label_opacity = theme
            .number_by_key(disabled_label_opacity_key)
            .unwrap_or(defaults.disabled_opacity);
        let icon_opacity = theme
            .number_by_key(disabled_icon_opacity_key)
            .unwrap_or(defaults.disabled_opacity);
        label.a = (label.a * label_opacity).clamp(0.0, 1.0);
        icon.a = (icon.a * icon_opacity).clamp(0.0, 1.0);
        opacity = 0.0;
    }

    (label, icon, state_layer, opacity)
}

fn list_item_ripple_base_opacity(theme: &Theme, selected: bool) -> f32 {
    theme
        .number_by_key(if selected {
            "md.comp.list.list-item.selected.pressed.state-layer.opacity"
        } else {
            "md.comp.list.list-item.pressed.state-layer.opacity"
        })
        .unwrap_or(0.1)
}

#[derive(Debug, Clone, Copy)]
enum ListIconSlot {
    Leading,
    Trailing,
}

fn list_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    icon: &IconId,
    color: Color,
    slot: ListIconSlot,
) -> AnyElement {
    let size_key = match slot {
        ListIconSlot::Leading => "md.comp.list.list-item.leading-icon.size",
        ListIconSlot::Trailing => "md.comp.list.list-item.trailing-icon.size",
    };
    let size = theme.metric_by_key(size_key).unwrap_or(Px(24.0));
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = fret_core::SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}

fn svg_source_for_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: &IconId) -> SvgSource {
    let resolved = cx
        .app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons
                .resolve_svg_owned(icon)
                .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}

fn list_item_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: &Arc<str>,
    color: Color,
) -> AnyElement {
    let style = theme
        .text_style_by_key("md.sys.typescale.body-large")
        .unwrap_or_else(TextStyle::default);

    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}
