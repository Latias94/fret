//! Material 3 input chip (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven shape/colors via `md.comp.input-chip.*` (v30 sassvars subset).
//! - Selection is represented as `WidgetStates::SELECTED`.
//! - State layer + bounded ripple using the shared Material foundation indication path.
//!
//! Note: Trailing icon can be upgraded to a dedicated nested pressable surface via
//! `InputChip::on_trailing_icon_activate`.

use std::sync::Arc;

use fret_core::{
    Axis, Color, Edges, LayoutDirection, Point, Px, Rect, SemanticsRole, Size, SvgFit,
    TextOverflow, TextStyle, TextWrap,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PositionStyle, PressableA11y, PressableProps, SpacerProps, SvgIconProps,
    TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::context::{resolved_layout_direction, theme_default_layout_direction};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable,
    material_ink_layer_for_pressable_with_ripple_bounds, material_pressable_indication_config,
};
use crate::foundation::interaction::pressable_interaction;
use crate::foundation::interactive_size::{
    centered_fill, enforce_minimum_interactive_size, minimum_interactive_size,
};
use crate::tokens::input_chip as input_chip_tokens;

#[derive(Debug, Clone, Default)]
pub struct InputChipStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub leading_icon_color: OverrideSlot<ColorRef>,
    pub trailing_icon_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl InputChipStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn leading_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.leading_icon_color = Some(color);
        self
    }

    pub fn trailing_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trailing_icon_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.leading_icon_color.is_some() {
            self.leading_icon_color = other.leading_icon_color;
        }
        if other.trailing_icon_color.is_some() {
            self.trailing_icon_color = other.trailing_icon_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct InputChip {
    label: Arc<str>,
    selected: Model<bool>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    on_activate: Option<OnActivate>,
    on_trailing_icon_activate: Option<OnActivate>,
    trailing_icon_a11y_label: Option<Arc<str>>,
    style: InputChipStyle,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    roving_tab_stop: Option<bool>,
}

impl std::fmt::Debug for InputChip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputChip")
            .field("label", &self.label)
            .field("selected", &"<model>")
            .field("leading_icon", &self.leading_icon)
            .field("trailing_icon", &self.trailing_icon)
            .field("on_activate", &self.on_activate.is_some())
            .field(
                "on_trailing_icon_activate",
                &self.on_trailing_icon_activate.is_some(),
            )
            .field("trailing_icon_a11y_label", &self.trailing_icon_a11y_label)
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("roving_tab_stop", &self.roving_tab_stop)
            .finish()
    }
}

impl InputChip {
    pub fn new(selected: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            selected,
            leading_icon: None,
            trailing_icon: None,
            on_activate: None,
            on_trailing_icon_activate: None,
            trailing_icon_a11y_label: None,
            style: InputChipStyle::default(),
            disabled: false,
            a11y_label: None,
            test_id: None,
            roving_tab_stop: None,
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

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn on_trailing_icon_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_trailing_icon_activate = Some(on_activate);
        self
    }

    pub fn trailing_icon_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.trailing_icon_a11y_label = Some(label.into());
        self
    }

    pub fn style(mut self, style: InputChipStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enable roving-focus-friendly tab stop behavior.
    ///
    /// When enabled, only the current tab stop (or the currently focused item) is included in the
    /// default focus traversal order.
    pub fn roving_tab_stop(mut self, tab_stop: bool) -> Self {
        self.roving_tab_stop = Some(tab_stop);
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

    pub(crate) fn disabled_for_roving(&self) -> bool {
        self.disabled
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                let focusable = match self.roving_tab_stop {
                    None => enabled,
                    Some(tab_stop) => enabled && (tab_stop || st.focused),
                };

                let selected_model_for_toggle = self.selected.clone();
                let enabled_for_toggle = enabled;
                let user_activate = self.on_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    if enabled_for_toggle {
                        let _ = host.update_model(&selected_model_for_toggle, |v| *v = !*v);
                        host.request_redraw(action_cx.window);
                    }
                    if let Some(h) = user_activate.as_ref() {
                        h(host, action_cx, reason);
                    }
                }));

                let checked = cx.get_model_copied(&self.selected, Invalidation::Layout);
                let now_frame = cx.frame_id.0;

                let (
                    corner_radii,
                    focus_ring,
                    height,
                    leading_icon_px,
                    trailing_icon_px,
                    default_layout_direction,
                    label_style,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let corner_radii = input_chip_tokens::container_shape(theme);
                    let focus_ring = material_focus_ring_for_component(
                        theme,
                        input_chip_tokens::COMPONENT_PREFIX,
                        corner_radii,
                    );
                    let height = input_chip_tokens::container_height(theme);
                    let leading_icon_px = input_chip_tokens::leading_icon_size(theme);
                    let trailing_icon_px = input_chip_tokens::trailing_icon_size(theme);
                    let default_layout_direction = theme_default_layout_direction(theme);
                    let label_style = theme
                        .text_style_by_key("md.sys.typescale.label-large")
                        .unwrap_or_else(TextStyle::default);
                    (
                        corner_radii,
                        focus_ring,
                        height,
                        leading_icon_px,
                        trailing_icon_px,
                        default_layout_direction,
                        label_style,
                    )
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: self.a11y_label.clone().or_else(|| Some(self.label.clone())),
                        test_id: self.test_id.clone(),
                        checked,
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        {
                            let theme = Theme::global(&*cx.app);
                            enforce_minimum_interactive_size(&mut l, theme);
                        }
                        l
                    },
                    focus_ring: Some(focus_ring),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    cx.pointer_region(props, |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let focus_visible =
                            fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                        let is_pressed = enabled && st.pressed;
                        let is_hovered = enabled && st.hovered;
                        let is_focused = enabled && st.focused && focus_visible;

                        let selected = cx
                            .get_model_copied(&self.selected, Invalidation::Paint)
                            .unwrap_or(false);

                        let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        if selected {
                            states |= WidgetStates::SELECTED;
                        }

                        let (
                            label_color,
                            leading_icon_color,
                            trailing_icon_color,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            indication_config,
                            background,
                            outline,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let label_color_default = input_chip_tokens::label_color(
                                theme,
                                selected,
                                enabled,
                                interaction,
                            );
                            let label_color = resolve_override_slot_with(
                                self.style.label_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || label_color_default,
                            );

                            let leading_icon_color_default = input_chip_tokens::leading_icon_color(
                                theme,
                                selected,
                                enabled,
                                interaction,
                            );
                            let leading_icon_color = resolve_override_slot_with(
                                self.style.leading_icon_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || leading_icon_color_default,
                            );

                            let trailing_icon_color_default =
                                input_chip_tokens::trailing_icon_color(
                                    theme,
                                    selected,
                                    enabled,
                                    interaction,
                                );
                            let trailing_icon_color = resolve_override_slot_with(
                                self.style.trailing_icon_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || trailing_icon_color_default,
                            );

                            let state_layer_color_default =
                                input_chip_tokens::state_layer_color(theme, selected, interaction);
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color_default,
                            );

                            let state_layer_target = input_chip_tokens::state_layer_opacity(
                                theme,
                                selected,
                                interaction,
                            );
                            let ripple_base_opacity =
                                input_chip_tokens::pressed_state_layer_opacity(theme, selected);
                            let indication_config =
                                material_pressable_indication_config(theme, None);

                            let background = if selected {
                                let bg_default = input_chip_tokens::selected_container_background(
                                    theme, enabled,
                                );
                                let bg = resolve_override_slot_with(
                                    self.style.container_background.as_ref(),
                                    states,
                                    |color| color.resolve(theme),
                                    || bg_default,
                                );
                                Some(bg)
                            } else {
                                None
                            };

                            let outline = if selected {
                                None
                            } else {
                                let outline_default = input_chip_tokens::unselected_outline(
                                    theme,
                                    enabled,
                                    interaction,
                                );
                                resolve_override_slot_opt_with(
                                    self.style.outline_color.as_ref(),
                                    states,
                                    |color| color.resolve(theme),
                                    || Some(outline_default.color),
                                )
                                .map(|color| {
                                    input_chip_tokens::ChipOutline {
                                        width: outline_default.width,
                                        color,
                                    }
                                })
                            };

                            (
                                label_color,
                                leading_icon_color,
                                trailing_icon_color,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                indication_config,
                                background,
                                outline,
                            )
                        };

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
                            false,
                        );

                        let leading_icon = self.leading_icon;
                        let leading_icon_size = leading_icon.as_ref().map(|_| leading_icon_px);
                        let trailing_icon = self.trailing_icon;
                        let trailing_icon_size = trailing_icon.as_ref().map(|_| trailing_icon_px);

                        let layout_direction =
                            resolved_layout_direction(cx, default_layout_direction);

                        let content = chip_content(
                            cx,
                            label_style.clone(),
                            &self.label,
                            label_color,
                            leading_icon,
                            leading_icon_size,
                            leading_icon_color,
                            trailing_icon,
                            trailing_icon_size,
                            trailing_icon_color,
                            enabled,
                            selected,
                            self.test_id.clone(),
                            self.on_trailing_icon_activate.clone(),
                            self.trailing_icon_a11y_label.clone(),
                            pressable_id,
                            layout_direction,
                            height,
                        );

                        let mut chrome = ContainerProps::default();
                        chrome.layout.overflow = Overflow::Visible;
                        chrome.corner_radii = corner_radii;
                        chrome.background = background;
                        chrome.layout.size.height = Length::Px(height);
                        if let Some(outline) = outline {
                            chrome.border = Edges::all(outline.width);
                            chrome.border_color = Some(outline.color);
                        }

                        let chrome = cx.container(chrome, move |_cx| vec![overlay, content]);

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn chip_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label_style: TextStyle,
    label: &Arc<str>,
    label_color: Color,
    leading_icon: Option<IconId>,
    leading_icon_size: Option<Px>,
    leading_icon_color: Color,
    trailing_icon: Option<IconId>,
    trailing_icon_size: Option<Px>,
    trailing_icon_color: Color,
    enabled: bool,
    selected: bool,
    chip_test_id: Option<Arc<str>>,
    on_trailing_icon_activate: Option<OnActivate>,
    trailing_icon_a11y_label: Option<Arc<str>>,
    primary_pressable_id: fret_ui::elements::GlobalElementId,
    layout_direction: LayoutDirection,
    height: Px,
) -> AnyElement {
    const LEADING_SPACE: Px = Px(16.0);
    const TRAILING_SPACE: Px = Px(16.0);
    const ICON_LABEL_SPACE: Px = Px(8.0);
    const WITH_LEADING_ICON_LEADING_SPACE: Px = Px(8.0);
    const WITH_TRAILING_ICON_TRAILING_SPACE: Px = Px(8.0);

    let mut text = TextProps::new(label.clone());
    text.style = Some(label_style);
    text.color = Some(label_color);
    text.wrap = TextWrap::None;
    text.overflow = TextOverflow::Ellipsis;

    let label_el = cx.text_props(text);

    let padding_left = if leading_icon.is_some() {
        WITH_LEADING_ICON_LEADING_SPACE
    } else {
        LEADING_SPACE
    };
    let padding_right = if trailing_icon.is_some() {
        WITH_TRAILING_ICON_TRAILING_SPACE
    } else {
        TRAILING_SPACE
    };

    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props.gap = Px(0.0);
    props.padding = Edges {
        left: padding_left,
        right: padding_right,
        top: Px(0.0),
        bottom: Px(0.0),
    };
    props.layout.size.height = Length::Px(height);
    props.layout.position = PositionStyle::Relative;

    cx.flex(props, move |cx| {
        let mut out = Vec::new();
        if let (Some(icon), Some(size)) = (leading_icon, leading_icon_size) {
            out.push(material_icon(cx, &icon, size, leading_icon_color));
            out.push(fixed_space(cx, ICON_LABEL_SPACE));
        }
        out.push(label_el);
        if let (Some(icon), Some(size)) = (trailing_icon, trailing_icon_size) {
            out.push(fixed_space(cx, ICON_LABEL_SPACE));
            out.push(material_icon(cx, &icon, size, trailing_icon_color));
            if let Some(handler) = on_trailing_icon_activate.clone() {
                out.push(trailing_icon_touch_target_overlay(
                    cx,
                    enabled,
                    selected,
                    chip_test_id.clone(),
                    label.clone(),
                    trailing_icon_a11y_label.clone(),
                    size,
                    handler,
                    primary_pressable_id,
                    layout_direction,
                    height,
                ));
            }
        }
        out
    })
}

fn fixed_space<H: UiHost>(cx: &mut ElementContext<'_, H>, width: Px) -> AnyElement {
    let mut props = SpacerProps::default();
    props.layout.flex.grow = 0.0;
    props.layout.flex.shrink = 0.0;
    props.layout.size.width = Length::Px(width);
    cx.spacer(props)
}

fn trailing_icon_touch_target_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    selected: bool,
    chip_test_id: Option<Arc<str>>,
    chip_label: Arc<str>,
    trailing_icon_a11y_label: Option<Arc<str>>,
    size: Px,
    on_activate: OnActivate,
    primary_pressable_id: fret_ui::elements::GlobalElementId,
    layout_direction: LayoutDirection,
    chip_height: Px,
) -> AnyElement {
    cx.named("trailing_icon_touch_target_overlay", move |cx| {
        cx.pressable_with_id_props(|cx, st, pressable_id| {
            cx.pressable_on_activate(on_activate.clone());

            let forward_key = match layout_direction {
                LayoutDirection::Ltr => fret_core::KeyCode::ArrowRight,
                LayoutDirection::Rtl => fret_core::KeyCode::ArrowLeft,
            };
            let backward_key = match layout_direction {
                LayoutDirection::Ltr => fret_core::KeyCode::ArrowLeft,
                LayoutDirection::Rtl => fret_core::KeyCode::ArrowRight,
            };

            let trailing_pressable_id = pressable_id;
            let primary_id = primary_pressable_id;

            cx.key_on_key_down_for(
                primary_id,
                Arc::new(move |host, _acx, key_cx| {
                    if key_cx.repeat {
                        return false;
                    }
                    if key_cx.modifiers.shift
                        || key_cx.modifiers.ctrl
                        || key_cx.modifiers.alt
                        || key_cx.modifiers.alt_gr
                        || key_cx.modifiers.meta
                    {
                        return false;
                    }
                    if key_cx.key != forward_key {
                        return false;
                    }
                    host.request_focus(trailing_pressable_id);
                    true
                }),
            );

            cx.key_on_key_down_for(
                trailing_pressable_id,
                Arc::new(move |host, _acx, key_cx| {
                    if key_cx.repeat {
                        return false;
                    }
                    if key_cx.modifiers.shift
                        || key_cx.modifiers.ctrl
                        || key_cx.modifiers.alt
                        || key_cx.modifiers.alt_gr
                        || key_cx.modifiers.meta
                    {
                        return false;
                    }
                    if key_cx.key != backward_key {
                        return false;
                    }
                    host.request_focus(primary_pressable_id);
                    true
                }),
            );

            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.overflow = Overflow::Visible;
            layout.position = PositionStyle::Absolute;

            // Match Material Web: trailing action gets a 48px tall "touch" target without changing
            // the visual chrome size/placement.
            let min_touch = {
                let theme = Theme::global(&*cx.app);
                minimum_interactive_size(theme)
            };
            let top = Px((chip_height.0 - min_touch.0) * 0.5);
            let width = Px(size.0 + 16.0);

            layout.inset.top = Some(top);
            // In flex layout, absolute positioning uses the content rect (excluding padding).
            // Offset by the chip's trailing padding so the touch target covers the visible edge.
            layout.inset.right = Some(Px(-8.0));
            layout.size.width = Length::Px(width);
            layout.size.height = Length::Px(min_touch);

            let test_id = chip_test_id
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}.trailing-icon")));
            let a11y_label = trailing_icon_a11y_label
                .clone()
                .unwrap_or_else(|| Arc::<str>::from(format!("Remove {chip_label}")));

            let pressable_props = PressableProps {
                enabled,
                focusable: false,
                key_activation: Default::default(),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(a11y_label),
                    test_id,
                    ..Default::default()
                },
                layout,
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

                    let focus_visible =
                        fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                    let is_pressed = enabled && st.pressed;
                    let is_hovered = enabled && st.hovered;
                    let is_focused = enabled && st.focused && focus_visible;

                    let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                    let now_frame = cx.frame_id.0;

                    let (
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        indication_config,
                    ) = {
                        let theme = Theme::global(&*cx.app);
                        let state_layer_color =
                            input_chip_tokens::state_layer_color(theme, selected, interaction);
                        let state_layer_target =
                            input_chip_tokens::state_layer_opacity(theme, selected, interaction);
                        let ripple_base_opacity =
                            input_chip_tokens::pressed_state_layer_opacity(theme, selected);
                        let indication_config = material_pressable_indication_config(theme, None);
                        (
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            indication_config,
                        )
                    };

                    let circle_size = Px(size.0 * (4.0 / 3.0));
                    let corner_radii = fret_core::Corners::all(Px(circle_size.0 / 2.0));
                    let paint_bounds = Rect::new(
                        Point::new(
                            Px((width.0 - circle_size.0) * 0.5),
                            Px((min_touch.0 - circle_size.0) * 0.5),
                        ),
                        Size::new(circle_size, circle_size),
                    );

                    let overlay = material_ink_layer_for_pressable_with_ripple_bounds(
                        cx,
                        pressable_id,
                        now_frame,
                        paint_bounds,
                        paint_bounds,
                        corner_radii,
                        RippleClip::Bounded,
                        state_layer_color,
                        is_pressed,
                        state_layer_target,
                        ripple_base_opacity,
                        indication_config,
                        false,
                    );

                    let mut chrome = ContainerProps::default();
                    chrome.layout.overflow = Overflow::Visible;
                    chrome.corner_radii = corner_radii;
                    chrome.layout.size.width = Length::Fill;
                    chrome.layout.size.height = Length::Fill;

                    let chrome = cx.container(chrome, move |_cx| vec![overlay]);
                    vec![chrome]
                })
            });

            (pressable_props, vec![pointer_region])
        })
    })
}

fn material_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}
