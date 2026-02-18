//! Material 3 switch (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.switch.*`.
//! - State layer (hover/pressed/focus) + ripple driven by `fret_ui::paint`.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Point, Px, Rect, SemanticsRole, Size, SvgFit, Transform2D,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable_with_ripple_bounds,
    material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;
use crate::tokens::switch as switch_tokens;

fn material_web_switch_handle_overshoot_ease(x: f32) -> f32 {
    // Material Web (M3) switch handle-container overshoot:
    // `transition: margin 300ms cubic-bezier(0.175, 0.885, 0.32, 1.275)`.
    fret_ui_kit::headless::easing::CubicBezier::new(0.175, 0.885, 0.32, 1.275).sample_unclamped(x)
}

#[derive(Debug, Clone, Default)]
pub struct SwitchStyle {
    pub track_color: OverrideSlot<ColorRef>,
    pub handle_color: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl SwitchStyle {
    pub fn track_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.track_color = Some(color);
        self
    }

    pub fn handle_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.handle_color = Some(color);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.track_color.is_some() {
            self.track_color = other.track_color;
        }
        if other.handle_color.is_some() {
            self.handle_color = other.handle_color;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Switch {
    selected: Model<bool>,
    disabled: bool,
    icons: bool,
    show_only_selected_icon: bool,
    icon_on: Option<IconId>,
    icon_off: Option<IconId>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
    style: SwitchStyle,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("disabled", &self.disabled)
            .field("icons", &self.icons)
            .field("show_only_selected_icon", &self.show_only_selected_icon)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .finish()
    }
}

impl Switch {
    pub fn new(selected: Model<bool>) -> Self {
        Self {
            selected,
            disabled: false,
            icons: false,
            show_only_selected_icon: false,
            icon_on: Some(fret_icons::ids::ui::CHECK),
            icon_off: Some(fret_icons::ids::ui::CLOSE),
            a11y_label: None,
            test_id: None,
            on_activate: None,
            style: SwitchStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows switch icons inside the thumb.
    ///
    /// This mirrors Material Web's `icons` property.
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// Shows only the "selected" icon when checked, and hides the unselected icon.
    ///
    /// This mirrors Material Web's `show-only-selected-icon` behavior and overrides `.icons(...)`.
    pub fn show_only_selected_icon(mut self, show_only_selected_icon: bool) -> Self {
        self.show_only_selected_icon = show_only_selected_icon;
        self
    }

    /// Overrides the icon displayed when the switch is selected.
    pub fn icon_on(mut self, icon: IconId) -> Self {
        self.icon_on = Some(icon);
        self
    }

    /// Overrides the icon displayed when the switch is unselected.
    pub fn icon_off(mut self, icon: IconId) -> Self {
        self.icon_off = Some(icon);
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

    /// Called after the switch toggles its `Model<bool>`.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: SwitchStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let size = {
                let theme = Theme::global(&*cx.app);
                switch_size_tokens(theme)
            };
            let icons_enabled = self.icons;
            let show_only_selected_icon = self.show_only_selected_icon;
            let icon_on = self.icon_on.clone();
            let icon_off = self.icon_off.clone();

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                cx.key_add_on_key_down_for(pressable_id, consume_enter_key_handler());

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

                let checked = cx
                    .get_model_copied(&self.selected, Invalidation::Layout)
                    .unwrap_or(false);

                let (corner_radii, layout, focus_ring) = {
                    let theme = Theme::global(&*cx.app);
                    let corner_radii = switch_tokens::state_layer_shape(theme);

                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.overflow = Overflow::Visible;
                    enforce_minimum_interactive_size(&mut layout, theme);

                    let focus_ring =
                        material_focus_ring_for_component(theme, "md.comp.switch", corner_radii);

                    (corner_radii, layout, focus_ring)
                };
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Switch),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(checked),
                        ..Default::default()
                    },
                    layout,
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
                        let is_focused_any = enabled && st.focused;
                        let is_focused_visible = is_focused_any && focus_visible;
                        let selected = cx
                            .get_model_copied(&self.selected, Invalidation::Paint)
                            .unwrap_or(false);

                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        if selected {
                            states |= WidgetStates::SELECTED;
                        }
                        let mut states_unselected = states;
                        states_unselected.remove(fret_ui_kit::WidgetState::Selected);
                        let mut states_selected = states;
                        states_selected.insert(fret_ui_kit::WidgetState::Selected);

                        let tokens_interaction_for = |is_focused: bool| match pressable_interaction(
                            is_pressed, is_hovered, is_focused,
                        ) {
                            Some(PressableInteraction::Pressed) => {
                                switch_tokens::SwitchInteraction::Pressed
                            }
                            Some(PressableInteraction::Focused) => {
                                switch_tokens::SwitchInteraction::Focused
                            }
                            Some(PressableInteraction::Hovered) => {
                                switch_tokens::SwitchInteraction::Hovered
                            }
                            None => switch_tokens::SwitchInteraction::None,
                        };

                        // Material Web uses mixed focus selectors:
                        // - handle + icons: `:focus-within`
                        // - track (unselected): `:focus-visible`
                        // - track (selected): `:focus-within`
                        //
                        // Mirror that split so mouse-focus can tint the handle/icons without
                        // forcing unselected track chroming.
                        let tokens_interaction_state_layer =
                            tokens_interaction_for(is_focused_visible);
                        let tokens_interaction_handle = tokens_interaction_for(is_focused_any);
                        let tokens_interaction_track_unselected =
                            tokens_interaction_for(is_focused_visible);
                        let tokens_interaction_track_selected =
                            tokens_interaction_for(is_focused_any);

                        #[derive(Default)]
                        struct SwitchThumbRuntime {
                            selected: SpringAnimator,
                            pressed: SpringAnimator,
                            prev_pressed: bool,
                        }

                        let (
                            state_layer_target,
                            state_layer_color,
                            spring,
                            chrome_unselected,
                            chrome_selected,
                            track_corner_radii,
                            handle_corner_radii,
                            ripple_base_opacity,
                            config,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let state_layer_target = switch_tokens::state_layer_target_opacity(
                                theme,
                                selected,
                                enabled,
                                tokens_interaction_state_layer,
                            );
                            let state_layer_color = switch_tokens::state_layer_color(
                                theme,
                                selected,
                                tokens_interaction_state_layer,
                            );
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let spring =
                                sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);

                            let chrome_unselected_track = switch_tokens::chrome(
                                theme,
                                false,
                                enabled,
                                tokens_interaction_track_unselected,
                            );
                            let chrome_unselected_handle = switch_tokens::chrome(
                                theme,
                                false,
                                enabled,
                                tokens_interaction_handle,
                            );
                            let mut chrome_unselected = switch_tokens::SwitchChrome {
                                track_color: chrome_unselected_track.track_color,
                                outline_color: chrome_unselected_track.outline_color,
                                handle_color: chrome_unselected_handle.handle_color,
                            };
                            let token_track_color = chrome_unselected.track_color;
                            chrome_unselected.track_color = resolve_override_slot_with(
                                self.style.track_color.as_ref(),
                                states_unselected,
                                |color| color.resolve(theme),
                                || token_track_color,
                            );
                            let token_handle_color = chrome_unselected.handle_color;
                            chrome_unselected.handle_color = resolve_override_slot_with(
                                self.style.handle_color.as_ref(),
                                states_unselected,
                                |color| color.resolve(theme),
                                || token_handle_color,
                            );
                            let token_outline_color = chrome_unselected.outline_color;
                            chrome_unselected.outline_color = resolve_override_slot_opt_with(
                                self.style.outline_color.as_ref(),
                                states_unselected,
                                |color| color.resolve(theme),
                                || token_outline_color,
                            );

                            let chrome_selected_track = switch_tokens::chrome(
                                theme,
                                true,
                                enabled,
                                tokens_interaction_track_selected,
                            );
                            let chrome_selected_handle = switch_tokens::chrome(
                                theme,
                                true,
                                enabled,
                                tokens_interaction_handle,
                            );
                            let mut chrome_selected = switch_tokens::SwitchChrome {
                                track_color: chrome_selected_track.track_color,
                                outline_color: chrome_selected_track.outline_color,
                                handle_color: chrome_selected_handle.handle_color,
                            };
                            let token_track_color = chrome_selected.track_color;
                            chrome_selected.track_color = resolve_override_slot_with(
                                self.style.track_color.as_ref(),
                                states_selected,
                                |color| color.resolve(theme),
                                || token_track_color,
                            );
                            let token_handle_color = chrome_selected.handle_color;
                            chrome_selected.handle_color = resolve_override_slot_with(
                                self.style.handle_color.as_ref(),
                                states_selected,
                                |color| color.resolve(theme),
                                || token_handle_color,
                            );
                            let token_outline_color = chrome_selected.outline_color;
                            chrome_selected.outline_color = resolve_override_slot_opt_with(
                                self.style.outline_color.as_ref(),
                                states_selected,
                                |color| color.resolve(theme),
                                || token_outline_color,
                            );

                            let track_corner_radii = switch_tokens::track_shape(theme);
                            let handle_corner_radii = switch_tokens::handle_shape(theme);

                            let ripple_base_opacity =
                                switch_tokens::pressed_state_layer_opacity(theme, selected);
                            let config = material_pressable_indication_config(
                                theme,
                                Some(Px(size.state_layer.0 * 0.5)),
                            );

                            (
                                state_layer_target,
                                state_layer_color,
                                spring,
                                chrome_unselected,
                                chrome_selected,
                                track_corner_radii,
                                handle_corner_radii,
                                ripple_base_opacity,
                                config,
                            )
                        };
                        let (thumb_t, pressed_t, chrome_t, position_t, thumb_active) =
                            cx.named("thumb_runtime", |cx| {
                                let desired_selected = if selected { 1.0 } else { 0.0 };
                                let desired_pressed = if is_pressed { 1.0 } else { 0.0 };

                                // Match Material Web's selected/unselected crossfade which is driven
                                // via pseudo-element opacity transitions (67ms linear).
                                let chrome_duration = if enabled {
                                    Duration::from_millis(67)
                                } else {
                                    Duration::ZERO
                                };
                                let chrome = fret_ui_kit::declarative::motion::drive_tween_f32(
                                    cx,
                                    desired_selected,
                                    chrome_duration,
                                    fret_ui_kit::headless::easing::linear,
                                );

                                // Match Material Web's handle-container "overshoot" position
                                // transition (margin 300ms cubic-bezier).
                                let position_duration = if enabled {
                                    Duration::from_millis(300)
                                } else {
                                    Duration::ZERO
                                };
                                let position =
                                    fret_ui_kit::declarative::motion::drive_tween_f32_unclamped(
                                        cx,
                                        desired_selected,
                                        position_duration,
                                        material_web_switch_handle_overshoot_ease,
                                    );

                                let thumb_state_id = cx.root_id();
                                cx.with_state_for(
                                    thumb_state_id,
                                    SwitchThumbRuntime::default,
                                    |rt| {
                                        if !rt.selected.is_initialized() {
                                            rt.selected.reset(now_frame, desired_selected);
                                        }
                                        if !rt.pressed.is_initialized() {
                                            rt.pressed.reset(now_frame, desired_pressed);
                                        }

                                        rt.selected.set_target(now_frame, desired_selected, spring);
                                        rt.pressed.set_target(now_frame, desired_pressed, spring);
                                        rt.selected.advance(now_frame);

                                        // Match Compose's `SnapSpec` for the "pressed" transition:
                                        // - snap to the pressed state on pointer down
                                        // - animate back to rest on release
                                        if is_pressed {
                                            if !rt.prev_pressed {
                                                rt.pressed.reset(now_frame, 1.0);
                                            }
                                        } else {
                                            rt.pressed.advance(now_frame);
                                        }
                                        rt.prev_pressed = is_pressed;
                                        (
                                            rt.selected.value(),
                                            rt.pressed.value(),
                                            chrome.value,
                                            position.value,
                                            rt.selected.is_active()
                                                || rt.pressed.is_active()
                                                || chrome.animating
                                                || position.animating,
                                        )
                                    },
                                )
                            });

                        let chrome =
                            mix_switch_chrome(chrome_unselected, chrome_selected, chrome_t);

                        let icons_always = icons_enabled && !show_only_selected_icon;
                        let icons_selected_only = show_only_selected_icon;
                        let geom = switch_geometry(
                            size,
                            thumb_t,
                            position_t,
                            pressed_t,
                            icons_always,
                            icons_selected_only,
                        );
                        let handle_child = material_switch_handle_icon(
                            cx,
                            thumb_t,
                            thumb_active,
                            selected,
                            enabled,
                            tokens_interaction_handle,
                            icons_enabled,
                            show_only_selected_icon,
                            icon_on.clone(),
                            icon_off.clone(),
                        );
                        let track = switch_track(
                            cx,
                            size,
                            geom,
                            chrome,
                            track_corner_radii,
                            handle_corner_radii,
                            handle_child,
                        );

                        let overlay = material_ink_layer_for_pressable_with_ripple_bounds(
                            cx,
                            pressable_id,
                            now_frame,
                            geom.ink_bounds,
                            geom.ink_bounds,
                            corner_radii,
                            RippleClip::Bounded,
                            state_layer_color,
                            is_pressed,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            thumb_active,
                        );

                        let mut outer = ContainerProps::default();
                        outer.layout.size.width = Length::Px(size.track_width);
                        outer.layout.size.height = Length::Px(size.state_layer);
                        outer.layout.overflow = Overflow::Visible;
                        outer.padding = Edges {
                            top: size.track_y_offset,
                            right: Px(0.0),
                            bottom: size.track_y_offset,
                            left: Px(0.0),
                        };
                        outer.corner_radii = Corners::all(Px(0.0));

                        let chrome = cx.container(outer, move |_cx| vec![overlay, track]);
                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct SwitchSizeTokens {
    state_layer: Px,
    track_width: Px,
    track_height: Px,
    track_outline_width: Px,
    selected_handle_width: Px,
    selected_handle_height: Px,
    unselected_handle_width: Px,
    unselected_handle_height: Px,
    pressed_handle_width: Px,
    pressed_handle_height: Px,
    with_icon_handle_width: Px,
    with_icon_handle_height: Px,
    track_y_offset: Px,
}

fn switch_size_tokens(theme: &Theme) -> SwitchSizeTokens {
    let state_layer = theme
        .metric_by_key("md.comp.switch.state-layer.size")
        .unwrap_or(Px(40.0));
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));
    let track_outline_width = theme
        .metric_by_key("md.comp.switch.track.outline.width")
        .unwrap_or(Px(2.0));

    let selected_handle_width = theme
        .metric_by_key("md.comp.switch.selected.handle.width")
        .unwrap_or(Px(24.0));
    let selected_handle_height = theme
        .metric_by_key("md.comp.switch.selected.handle.height")
        .unwrap_or(Px(24.0));
    let unselected_handle_width = theme
        .metric_by_key("md.comp.switch.unselected.handle.width")
        .unwrap_or(Px(16.0));
    let unselected_handle_height = theme
        .metric_by_key("md.comp.switch.unselected.handle.height")
        .unwrap_or(Px(16.0));
    let pressed_handle_width = theme
        .metric_by_key("md.comp.switch.pressed.handle.width")
        .unwrap_or(Px(28.0));
    let pressed_handle_height = theme
        .metric_by_key("md.comp.switch.pressed.handle.height")
        .unwrap_or(Px(28.0));

    let with_icon_handle_width = theme
        .metric_by_key("md.comp.switch.with-icon.handle.width")
        .unwrap_or(selected_handle_width);
    let with_icon_handle_height = theme
        .metric_by_key("md.comp.switch.with-icon.handle.height")
        .unwrap_or(selected_handle_height);

    let track_y_offset = Px(((state_layer.0 - track_height.0) * 0.5).max(0.0));

    SwitchSizeTokens {
        state_layer,
        track_width,
        track_height,
        track_outline_width,
        selected_handle_width,
        selected_handle_height,
        unselected_handle_width,
        unselected_handle_height,
        pressed_handle_width,
        pressed_handle_height,
        with_icon_handle_width,
        with_icon_handle_height,
        track_y_offset,
    }
}

#[derive(Debug, Clone, Copy)]
struct SwitchGeometry {
    handle_x: Px,
    handle_y: Px,
    handle_width: Px,
    handle_height: Px,
    ink_bounds: Rect,
}

fn switch_geometry(
    size: SwitchSizeTokens,
    size_t: f32,
    position_t: f32,
    pressed: f32,
    icons_always: bool,
    icons_selected_only: bool,
) -> SwitchGeometry {
    let size_t = size_t.clamp(0.0, 1.0);
    // Allow a small overshoot to match Material Web's custom cubic-bezier position easing.
    let position_t = position_t.clamp(-0.2, 1.2);

    let pressed_t = pressed.clamp(0.0, 1.0);
    let unselected_width = if icons_always {
        size.with_icon_handle_width
    } else {
        size.unselected_handle_width
    };
    let unselected_height = if icons_always {
        size.with_icon_handle_height
    } else {
        size.unselected_handle_height
    };
    let selected_width = if icons_always || icons_selected_only {
        size.with_icon_handle_width
    } else {
        size.selected_handle_width
    };
    let selected_height = if icons_always || icons_selected_only {
        size.with_icon_handle_height
    } else {
        size.selected_handle_height
    };

    let base_width = Px(unselected_width.0 + (selected_width.0 - unselected_width.0) * size_t);
    let base_height = Px(unselected_height.0 + (selected_height.0 - unselected_height.0) * size_t);

    let handle_width = Px(base_width.0 + (size.pressed_handle_width.0 - base_width.0) * pressed_t);
    let handle_height =
        Px(base_height.0 + (size.pressed_handle_height.0 - base_height.0) * pressed_t);

    // Material Web switch uses a circular handle; keep symmetric padding behavior and derive it
    // from the handle height (the primary axis for vertical centering).
    let padding_y = Px(((size.track_height.0 - handle_height.0) * 0.5).max(0.0));
    let padding_x = padding_y;

    let on_x = Px(size.track_width.0 - handle_width.0 - padding_x.0);
    let off_x = padding_x;
    let handle_x = Px(off_x.0 + (on_x.0 - off_x.0) * position_t);
    let handle_y = padding_y;

    let thumb_cx = Px(handle_x.0 + handle_width.0 * 0.5);
    let thumb_cy = Px(size.track_y_offset.0 + handle_y.0 + handle_height.0 * 0.5);

    let ink_origin = fret_core::Point::new(
        Px(thumb_cx.0 - size.state_layer.0 * 0.5),
        Px(thumb_cy.0 - size.state_layer.0 * 0.5),
    );
    let ink_bounds = Rect::new(ink_origin, Size::new(size.state_layer, size.state_layer));

    SwitchGeometry {
        handle_x,
        handle_y,
        handle_width,
        handle_height,
        ink_bounds,
    }
}

fn switch_track<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: SwitchSizeTokens,
    geom: SwitchGeometry,
    chrome: switch_tokens::SwitchChrome,
    track_corner_radii: Corners,
    handle_corner_radii: Corners,
    handle_child: Option<AnyElement>,
) -> AnyElement {
    let mut track = ContainerProps::default();
    track.layout.size.width = Length::Px(size.track_width);
    track.layout.size.height = Length::Px(size.track_height);
    track.layout.overflow = Overflow::Visible;
    track.background = Some(chrome.track_color);
    track.corner_radii = track_corner_radii;
    if let Some(outline) = chrome.outline_color {
        track.border = Edges::all(size.track_outline_width);
        track.border_color = Some(outline);
    }

    cx.container(track, move |cx| {
        let mut handle = ContainerProps::default();
        handle.layout.position = fret_ui::element::PositionStyle::Absolute;
        handle.layout.inset.left = Some(geom.handle_x);
        handle.layout.inset.top = Some(geom.handle_y);
        handle.layout.size.width = Length::Px(geom.handle_width);
        handle.layout.size.height = Length::Px(geom.handle_height);
        handle.corner_radii = handle_corner_radii;
        handle.background = Some(chrome.handle_color);

        vec![cx.container(handle, move |cx| {
            let Some(child) = handle_child else {
                return Vec::new();
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            vec![cx.flex(
                FlexProps {
                    layout,
                    direction: Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| vec![child],
            )]
        })]
    })
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn mix_switch_chrome(
    unselected: switch_tokens::SwitchChrome,
    selected: switch_tokens::SwitchChrome,
    t: f32,
) -> switch_tokens::SwitchChrome {
    let t = t.clamp(0.0, 1.0);

    let outline_color = match (unselected.outline_color, selected.outline_color) {
        (Some(a), Some(b)) => Some(lerp_color(a, b, t)),
        (Some(a), None) => Some(alpha_mul(a, 1.0 - t)),
        (None, Some(b)) => Some(alpha_mul(b, t)),
        (None, None) => None,
    };

    switch_tokens::SwitchChrome {
        track_color: lerp_color(unselected.track_color, selected.track_color, t),
        outline_color,
        handle_color: lerp_color(unselected.handle_color, selected.handle_color, t),
    }
}

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}

fn material_switch_handle_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    thumb_t: f32,
    thumb_active: bool,
    selected: bool,
    enabled: bool,
    interaction: switch_tokens::SwitchInteraction,
    icons: bool,
    show_only_selected_icon: bool,
    icon_on: Option<IconId>,
    icon_off: Option<IconId>,
) -> Option<AnyElement> {
    let thumb_t = thumb_t.clamp(0.0, 1.0);

    if show_only_selected_icon {
        if !thumb_active && !selected {
            return None;
        }

        let on_icon = icon_on?;
        let (size, color) = {
            let theme = Theme::global(&*cx.app);
            let size = switch_tokens::icon_size(theme, true);
            let color = switch_tokens::icon_color(theme, true, enabled, interaction);
            (size, color)
        };
        let opacity = thumb_t;
        let rotation_degrees = -45.0 * (1.0 - thumb_t);

        let layer =
            material_switch_icon_layer(cx, &on_icon, size, color, opacity, rotation_degrees);
        return Some(material_switch_icon_overlay(cx, vec![layer]));
    }

    if !icons {
        return None;
    }

    let on_icon = icon_on?;
    let off_icon = icon_off?;
    let (on, off) = {
        let theme = Theme::global(&*cx.app);
        let on_size = switch_tokens::icon_size(theme, true);
        let off_size = switch_tokens::icon_size(theme, false);
        let on_color = switch_tokens::icon_color(theme, true, enabled, interaction);
        let off_color = switch_tokens::icon_color(theme, false, enabled, interaction);
        ((on_size, on_color), (off_size, off_color))
    };

    let on_layer = material_switch_icon_layer(cx, &on_icon, on.0, on.1, thumb_t, 0.0);
    let off_layer = material_switch_icon_layer(cx, &off_icon, off.0, off.1, 1.0 - thumb_t, 0.0);
    Some(material_switch_icon_overlay(cx, vec![on_layer, off_layer]))
}

fn material_switch_icon_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layers: Vec<AnyElement>,
) -> AnyElement {
    let mut wrapper = ContainerProps::default();
    wrapper.layout.size.width = Length::Fill;
    wrapper.layout.size.height = Length::Fill;
    wrapper.layout.position = fret_ui::element::PositionStyle::Relative;
    wrapper.layout.overflow = Overflow::Visible;
    cx.container(wrapper, move |_cx| layers)
}

fn material_switch_icon_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
    opacity: f32,
    rotation_degrees: f32,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    props.opacity = opacity.clamp(0.0, 1.0);

    let center = Point::new(Px(size.0 * 0.5), Px(size.0 * 0.5));
    let transform = Transform2D::rotation_about_degrees(rotation_degrees, center);

    let icon = cx.visual_transform(transform, move |cx| vec![cx.svg_icon_props(props)]);

    let mut layer = ContainerProps::default();
    layer.layout.position = fret_ui::element::PositionStyle::Absolute;
    layer.layout.inset.top = Some(Px(0.0));
    layer.layout.inset.right = Some(Px(0.0));
    layer.layout.inset.bottom = Some(Px(0.0));
    layer.layout.inset.left = Some(Px(0.0));
    layer.layout.size.width = Length::Fill;
    layer.layout.size.height = Length::Fill;
    layer.layout.overflow = Overflow::Visible;
    cx.container(layer, move |cx| {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        vec![cx.flex(
            FlexProps {
                layout,
                direction: Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![icon],
        )]
    })
}
