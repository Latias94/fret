//! Material 3 list (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing and colors via `md.comp.list.*` (subset).
//! - Roving focus + automatic activation (selection follows focus).
//! - State layer + bounded ripple aligned to item bounds.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
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

use crate::foundation::context::{
    MaterialDesignVariant, resolved_design_variant, theme_default_design_variant,
};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::tokens::list as list_tokens;

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

        let design_variant = resolved_design_variant(cx, theme_default_design_variant(theme));
        let expressive = design_variant == MaterialDesignVariant::Expressive;

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

        let height = list_tokens::one_line_container_height(theme);

        let focus_ring_corner_radii = list_tokens::item_container_shape_for_interaction(
            theme,
            selected,
            enabled,
            list_tokens::ListItemInteraction::Focused,
            expressive,
        );

        let focus_ring =
            material_focus_ring_for_component(theme, "md.comp.list", focus_ring_corner_radii);

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
                    list_tokens::item_outcomes(theme, selected, enabled, interaction);

                let corner_radii = list_tokens::item_container_shape_for_interaction(
                    theme,
                    selected,
                    enabled,
                    interaction,
                    expressive,
                );

                let ripple_base_opacity = list_tokens::pressed_state_layer_opacity(theme, selected);
                let config = material_pressable_indication_config(theme, None);
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
                    config,
                    false,
                );

                let selected_bg = if selected {
                    Some(list_tokens::selected_container_background(theme, enabled))
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
                row.gap = list_tokens::item_between_space(theme);
                row.padding = Edges {
                    left: list_tokens::item_leading_space(theme),
                    right: list_tokens::item_trailing_space(theme),
                    top: list_tokens::item_top_space(theme),
                    bottom: list_tokens::item_bottom_space(theme),
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
                        let leading = leading_icon.as_ref().map(|id| {
                            list_icon(cx, theme, id, icon_color, expressive, ListIconSlot::Leading)
                        });
                        let trailing = trailing_icon.as_ref().map(|id| {
                            list_icon(
                                cx,
                                theme,
                                id,
                                icon_color,
                                expressive,
                                ListIconSlot::Trailing,
                            )
                        });
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

fn interaction_state(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> list_tokens::ListItemInteraction {
    match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => list_tokens::ListItemInteraction::Pressed,
        Some(PressableInteraction::Focused) => list_tokens::ListItemInteraction::Focused,
        Some(PressableInteraction::Hovered) => list_tokens::ListItemInteraction::Hovered,
        None => list_tokens::ListItemInteraction::Default,
    }
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
    expressive: bool,
    slot: ListIconSlot,
) -> AnyElement {
    let size = match slot {
        ListIconSlot::Leading => list_tokens::leading_icon_size_with_variant(theme, expressive),
        ListIconSlot::Trailing => list_tokens::trailing_icon_size_with_variant(theme, expressive),
    };
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
