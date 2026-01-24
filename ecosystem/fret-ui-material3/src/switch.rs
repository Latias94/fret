//! Material 3 switch (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.switch.*`.
//! - State layer (hover/pressed/focus) + unbounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, KeyCode, Px, Rect, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, Overflow, PointerRegionProps, PressableA11y, PressableProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable_with_ripple_bounds,
    material_pressable_indication_config,
};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;

#[derive(Clone)]
pub struct Switch {
    selected: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
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

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                        let state_layer_target = switch_state_layer_target_opacity(
                            &theme,
                            selected,
                            enabled,
                            interaction,
                        );
                        let state_layer_color =
                            switch_state_layer_color(&theme, selected, interaction);

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
                        let track =
                            switch_track(cx, &theme, size, selected, enabled, interaction, geom);

                        let ripple_base_opacity = switch_ripple_base_opacity(&theme, selected);
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
                            RippleClip::Unbounded,
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

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
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

fn switch_state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let group = if selected { "selected" } else { "unselected" };
    match interaction {
        Interaction::Pressed => theme
            .number_by_key(&format!(
                "md.comp.switch.{group}.pressed.state-layer.opacity"
            ))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Focused => theme
            .number_by_key(&format!("md.comp.switch.{group}.focus.state-layer.opacity"))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        Interaction::Hovered => theme
            .number_by_key(&format!("md.comp.switch.{group}.hover.state-layer.opacity"))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
        Interaction::None => 0.0,
    }
}

fn switch_ripple_base_opacity(theme: &Theme, selected: bool) -> f32 {
    let group = if selected { "selected" } else { "unselected" };
    theme
        .number_by_key(&format!(
            "md.comp.switch.{group}.pressed.state-layer.opacity"
        ))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

fn switch_state_layer_color(theme: &Theme, selected: bool, interaction: Interaction) -> Color {
    let group = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Interaction::Pressed => format!("md.comp.switch.{group}.pressed.state-layer.color"),
        Interaction::Focused => format!("md.comp.switch.{group}.focus.state-layer.color"),
        Interaction::Hovered => format!("md.comp.switch.{group}.hover.state-layer.color"),
        Interaction::None => format!("md.comp.switch.{group}.hover.state-layer.color"),
    };

    theme.color_by_key(&key).unwrap_or_else(|| {
        theme
            .color_by_key("md.sys.color.primary")
            .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
    })
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
    theme: &Theme,
    size: SwitchSizeTokens,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
    geom: SwitchGeometry,
) -> AnyElement {
    let colors = switch_chrome(theme, selected, enabled, interaction);

    let mut track = ContainerProps::default();
    track.layout.size.width = Length::Px(size.track_width);
    track.layout.size.height = Length::Px(size.track_height);
    track.layout.overflow = Overflow::Visible;
    track.background = Some(colors.track_color);
    track.corner_radii = Corners::all(Px(9999.0));
    if let Some(outline) = colors.outline_color {
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
        handle.background = Some(colors.handle_color);

        vec![cx.container(handle, |_cx| Vec::new())]
    })
}

#[derive(Debug, Clone, Copy)]
struct SwitchChrome {
    track_color: Color,
    outline_color: Option<Color>,
    handle_color: Color,
}

fn switch_chrome(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Interaction,
) -> SwitchChrome {
    if !enabled {
        let track_base = if selected {
            theme.color_by_key("md.comp.switch.disabled.selected.track.color")
        } else {
            theme.color_by_key("md.comp.switch.disabled.unselected.track.color")
        }
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

        let track_opacity = theme
            .number_by_key("md.comp.switch.disabled.track.opacity")
            .unwrap_or(0.12);
        let track_color = alpha_mul(track_base, track_opacity);

        let handle_base = if selected {
            theme
                .color_by_key("md.comp.switch.disabled.selected.handle.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface"))
        } else {
            theme
                .color_by_key("md.comp.switch.disabled.unselected.handle.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        }
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

        let handle_opacity = if selected {
            theme.number_by_key("md.comp.switch.disabled.selected.handle.opacity")
        } else {
            theme.number_by_key("md.comp.switch.disabled.unselected.handle.opacity")
        }
        .unwrap_or(0.38);
        let handle_color = alpha_mul(handle_base, handle_opacity);

        let outline_color = if selected {
            None
        } else {
            theme
                .color_by_key("md.comp.switch.disabled.unselected.track.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                .map(|c| alpha_mul(c, handle_opacity))
        };

        return SwitchChrome {
            track_color,
            outline_color,
            handle_color,
        };
    }

    let group = if selected { "selected" } else { "unselected" };
    let state = match interaction {
        Interaction::Pressed => "pressed",
        Interaction::Focused => "focus",
        Interaction::Hovered => "hover",
        Interaction::None => "",
    };

    let track_key = if state.is_empty() {
        format!("md.comp.switch.{group}.track.color")
    } else {
        format!("md.comp.switch.{group}.{state}.track.color")
    };
    let handle_key = if state.is_empty() {
        format!("md.comp.switch.{group}.handle.color")
    } else {
        format!("md.comp.switch.{group}.{state}.handle.color")
    };

    let track_color = theme.color_by_key(&track_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
        } else {
            theme
                .color_by_key("md.sys.color.surface-container-highest")
                .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container-highest"))
        }
    });

    let handle_color = theme.color_by_key(&handle_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.on-primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.on-primary"))
        } else {
            theme
                .color_by_key("md.sys.color.outline")
                .unwrap_or_else(|| theme.color_required("md.sys.color.outline"))
        }
    });

    let outline_color = if selected {
        None
    } else {
        let outline_key = if state.is_empty() {
            "md.comp.switch.unselected.track.outline.color".to_string()
        } else {
            format!("md.comp.switch.unselected.{state}.track.outline.color")
        };
        Some(
            theme
                .color_by_key(&outline_key)
                .or_else(|| theme.color_by_key("md.sys.color.outline"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.outline")),
        )
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}
