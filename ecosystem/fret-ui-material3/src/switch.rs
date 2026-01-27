//! Material 3 switch (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.switch.*`.
//! - State layer (hover/pressed/focus) + unbounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Corners, Edges, KeyCode, Px, Rect, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, Overflow, PointerRegionProps, PressableA11y, PressableProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable_with_ripple_bounds,
    material_pressable_indication_config,
};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;
use crate::tokens::switch as switch_tokens;

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
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
    style: SwitchStyle,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("disabled", &self.disabled)
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let size = switch_size_tokens(&theme);

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

                let corner_radii = Corners::all(Px(9999.0));
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Switch),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(
                            cx.get_model_copied(&self.selected, Invalidation::Layout)
                                .unwrap_or(false),
                        ),
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
                        "md.comp.switch",
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
                        let selected = cx
                            .get_model_copied(&self.selected, Invalidation::Paint)
                            .unwrap_or(false);

                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        if selected {
                            states |= WidgetStates::SELECTED;
                        }

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                        let tokens_interaction = match interaction {
                            Interaction::None => switch_tokens::SwitchInteraction::None,
                            Interaction::Hovered => switch_tokens::SwitchInteraction::Hovered,
                            Interaction::Focused => switch_tokens::SwitchInteraction::Focused,
                            Interaction::Pressed => switch_tokens::SwitchInteraction::Pressed,
                        };

                        let state_layer_target = switch_tokens::state_layer_target_opacity(
                            &theme,
                            selected,
                            enabled,
                            tokens_interaction,
                        );
                        let state_layer_color =
                            switch_tokens::state_layer_color(&theme, selected, tokens_interaction);
                        let state_layer_color = resolve_override_slot_with(
                            self.style.state_layer_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || state_layer_color,
                        );

                        #[derive(Default)]
                        struct SwitchThumbRuntime {
                            selected: SpringAnimator,
                            pressed: SpringAnimator,
                        }

                        let spring =
                            sys_spring_in_scope(&*cx, &theme, MotionSchemeKey::FastSpatial);
                        let (thumb_t, pressed_t, thumb_active) =
                            cx.with_state_for(pressable_id, SwitchThumbRuntime::default, |rt| {
                                let desired_selected = if selected { 1.0 } else { 0.0 };
                                let desired_pressed = if is_pressed { 1.0 } else { 0.0 };

                                if !rt.selected.is_initialized() {
                                    rt.selected.reset(now_frame, desired_selected);
                                }
                                if !rt.pressed.is_initialized() {
                                    rt.pressed.reset(now_frame, desired_pressed);
                                }

                                rt.selected.set_target(now_frame, desired_selected, spring);
                                rt.pressed.set_target(now_frame, desired_pressed, spring);
                                rt.selected.advance(now_frame);
                                rt.pressed.advance(now_frame);
                                (
                                    rt.selected.value(),
                                    rt.pressed.value(),
                                    rt.selected.is_active() || rt.pressed.is_active(),
                                )
                            });

                        let geom = switch_geometry(size, thumb_t, pressed_t);
                        let mut chrome =
                            switch_tokens::chrome(&theme, selected, enabled, tokens_interaction);
                        let token_track_color = chrome.track_color;
                        chrome.track_color = resolve_override_slot_with(
                            self.style.track_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || token_track_color,
                        );
                        let token_handle_color = chrome.handle_color;
                        chrome.handle_color = resolve_override_slot_with(
                            self.style.handle_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || token_handle_color,
                        );
                        let token_outline_color = chrome.outline_color;
                        chrome.outline_color = resolve_override_slot_opt_with(
                            self.style.outline_color.as_ref(),
                            states,
                            |color| color.resolve(&theme),
                            || token_outline_color,
                        );
                        let track = switch_track(cx, size, geom, chrome);

                        let ripple_base_opacity =
                            switch_tokens::pressed_state_layer_opacity(&theme, selected);
                        let config = material_pressable_indication_config(
                            &theme,
                            Some(Px(size.state_layer.0 * 0.5)),
                        );
                        let overlay = material_ink_layer_for_pressable_with_ripple_bounds(
                            cx,
                            pressable_id,
                            now_frame,
                            geom.ink_bounds,
                            geom.ink_bounds,
                            Corners::all(Px(9999.0)),
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

fn switch_geometry(size: SwitchSizeTokens, thumb_t: f32, pressed: f32) -> SwitchGeometry {
    let thumb_t = thumb_t.clamp(0.0, 1.0);

    let pressed_t = pressed.clamp(0.0, 1.0);
    let base_width = Px(size.unselected_handle_width.0
        + (size.selected_handle_width.0 - size.unselected_handle_width.0) * thumb_t);
    let base_height = Px(size.unselected_handle_height.0
        + (size.selected_handle_height.0 - size.unselected_handle_height.0) * thumb_t);

    let handle_width = Px(base_width.0 + (size.pressed_handle_width.0 - base_width.0) * pressed_t);
    let handle_height =
        Px(base_height.0 + (size.pressed_handle_height.0 - base_height.0) * pressed_t);

    // Material Web switch uses a circular handle; keep symmetric padding behavior and derive it
    // from the handle height (the primary axis for vertical centering).
    let padding_y = Px(((size.track_height.0 - handle_height.0) * 0.5).max(0.0));
    let padding_x = padding_y;

    let on_x = Px(size.track_width.0 - handle_width.0 - padding_x.0);
    let off_x = padding_x;
    let handle_x = Px(off_x.0 + (on_x.0 - off_x.0) * thumb_t);
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
) -> AnyElement {
    let mut track = ContainerProps::default();
    track.layout.size.width = Length::Px(size.track_width);
    track.layout.size.height = Length::Px(size.track_height);
    track.layout.overflow = Overflow::Visible;
    track.background = Some(chrome.track_color);
    track.corner_radii = Corners::all(Px(9999.0));
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
        handle.corner_radii = Corners::all(Px(9999.0));
        handle.background = Some(chrome.handle_color);

        vec![cx.container(handle, |_cx| Vec::new())]
    })
}

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}
