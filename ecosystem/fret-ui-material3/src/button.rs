//! Material 3 button (MVP).
//!
//! This starts with outcome alignment:
//! - State layer (hover/pressed/focus) using `fret_ui::paint::paint_state_layer`.
//! - Bounded ripple using `fret_ui::paint::paint_ripple`.
//!
//! The state machines live in this crate; the renderer-facing primitives remain mechanism-only in
//! `crates/fret-ui`.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::{SpringAnimator, SpringSpec};
use crate::tokens::button as button_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Elevated,
    Outlined,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Small,
}

#[derive(Debug, Clone, Default)]
pub struct ButtonStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl ButtonStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
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
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
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
pub struct Button {
    label: Arc<str>,
    variant: ButtonVariant,
    size: ButtonSize,
    on_activate: Option<OnActivate>,
    style: ButtonStyle,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Button {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            on_activate: None,
            style: ButtonStyle::default(),
            disabled: false,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let pressed = enabled && st.pressed;
                let (base_radius, pressed_radius, corner_spring, size_tokens, label_style) = {
                    let theme = Theme::global(&*cx.app);
                    let base_radius = button_shape_radius(theme);
                    let pressed_radius = button_pressed_shape_radius(theme);
                    let scheme_spring =
                        sys_spring_in_scope(&*cx, theme, MotionSchemeKey::FastSpatial);
                    let corner_spring = button_pressed_corner_spring(theme, scheme_spring);
                    let size_tokens = button_size_tokens(theme, self.size);
                    let label_style = button_label_style(theme);
                    (
                        base_radius,
                        pressed_radius,
                        corner_spring,
                        size_tokens,
                        label_style,
                    )
                };

                let (corner_radii, corner_want_frames) = animated_button_corner_radii(
                    cx,
                    pressable_id,
                    now_frame,
                    pressed,
                    base_radius,
                    pressed_radius,
                    corner_spring,
                );

                let focus_ring = {
                    let theme = Theme::global(&*cx.app);
                    material_focus_ring_for_component(theme, "md.comp.button", corner_radii)
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(self.label.clone()),
                        test_id: self.test_id.clone(),
                        ..Default::default()
                    },
                    // Keep the pressable overflow visible so focus rings can extend outward.
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l
                    },
                    focus_ring: Some(focus_ring),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    // PointerRegion is used to record `PointerRegionState.last_down` so ripple
                    // origins can align to pointer position without custom hook plumbing.
                    cx.pointer_region(props, |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let focus_visible =
                            fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                        let is_pressed = enabled && st.pressed;
                        let is_hovered = enabled && st.hovered;
                        let is_focused = enabled && st.focused && focus_visible;

                        let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                        let token_interaction = interaction.map(|s| match s {
                            PressableInteraction::Hovered => {
                                button_tokens::ButtonInteraction::Hovered
                            }
                            PressableInteraction::Focused => {
                                button_tokens::ButtonInteraction::Focused
                            }
                            PressableInteraction::Pressed => {
                                button_tokens::ButtonInteraction::Pressed
                            }
                        });

                        let states = WidgetStates::from_pressable(cx, st, enabled);

                        let (
                            label_color,
                            container_bg,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            outline,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let label_color = resolve_override_slot_with(
                                self.style.label_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || button_tokens::label_color(theme, self.variant, enabled),
                            );
                            let container_bg = button_tokens::container_background(
                                theme,
                                self.variant,
                                enabled,
                                label_color,
                            );
                            let container_bg = resolve_override_slot_opt_with(
                                self.style.container_background.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || container_bg,
                            );
                            let state_layer_color = button_tokens::state_layer_color(
                                theme,
                                self.variant,
                                label_color,
                                token_interaction,
                            );
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );
                            let state_layer_target = token_interaction
                                .map(|i| button_tokens::state_layer_opacity(theme, self.variant, i))
                                .unwrap_or(0.0);
                            let ripple_base_opacity =
                                button_tokens::pressed_state_layer_opacity(theme, self.variant);
                            let config = material_pressable_indication_config(theme, None);

                            let outline = button_outline(theme, self.variant, enabled);
                            let outline = outline.map(|mut outline| {
                                outline.color = resolve_override_slot_with(
                                    self.style.outline_color.as_ref(),
                                    states,
                                    |color| color.resolve(theme),
                                    || outline.color,
                                );
                                outline
                            });

                            (
                                label_color,
                                container_bg,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                config,
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
                            config,
                            corner_want_frames,
                        );

                        let label =
                            material_button_label(cx, label_style, &self.label, label_color);
                        let content = material_button_content(cx, size_tokens, label);

                        let chrome = material_button_chrome(
                            cx,
                            container_bg,
                            corner_radii,
                            outline,
                            size_tokens,
                            vec![overlay, content],
                        );

                        vec![chrome]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn material_button_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    background: Option<Color>,
    corner_radii: Corners,
    outline: Option<ButtonOutline>,
    size: ButtonSizeTokens,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.background = background;
    props.corner_radii = corner_radii;
    props.layout.size.min_height = Some(size.container_height);
    if let Some(outline) = outline {
        props.border = Edges::all(outline.width);
        props.border_color = Some(outline.color);
    }

    cx.container(props, move |_cx| children)
}

fn material_button_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: fret_core::TextStyle,
    label: &Arc<str>,
    color: Color,
) -> AnyElement {
    let mut props = TextProps::new(label.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    cx.text_props(props)
}

fn material_button_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: ButtonSizeTokens,
    child: AnyElement,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    // The outer container has no padding so the ink layer (absolute) can cover the full button.
    // Apply padding here instead.
    layout.size.height = Length::Px(size.container_height);
    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges {
                left: size.leading_space,
                right: size.trailing_space,
                top: Px(0.0),
                bottom: Px(0.0),
            },
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![child],
    )
}

#[derive(Debug, Default, Clone)]
struct ButtonCornerRuntime {
    spring: SpringAnimator,
}

fn button_label_style(theme: &Theme) -> fret_core::TextStyle {
    theme
        .text_style_by_key("md.comp.button.label-text")
        .or_else(|| theme.text_style_by_key("md.sys.typescale.label-large"))
        .or_else(|| theme.text_style_by_key("text_style.button"))
        .unwrap_or_else(|| fret_core::TextStyle::default())
}

fn button_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.button.container.shape.round")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(999.0))
        .0
}

fn button_pressed_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.button.pressed.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Px(8.0))
        .0
}

fn button_pressed_corner_spring(theme: &Theme, scheme_fallback: SpringSpec) -> SpringSpec {
    let tokens = MaterialTokenResolver::new(theme);
    SpringSpec {
        damping: tokens.number_comp_or_sys(
            "md.comp.button.pressed.container.corner-size.motion.spring.damping",
            "md.sys.motion.spring.fast.spatial.damping",
            scheme_fallback.damping,
        ),
        stiffness: tokens.number_comp_or_sys(
            "md.comp.button.pressed.container.corner-size.motion.spring.stiffness",
            "md.sys.motion.spring.fast.spatial.stiffness",
            scheme_fallback.stiffness,
        ),
    }
}

fn animated_button_corner_radii<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    pressed: bool,
    base_radius: f32,
    pressed_radius: f32,
    spring: SpringSpec,
) -> (Corners, bool) {
    let target = if pressed { pressed_radius } else { base_radius };

    cx.with_state_for(pressable_id, ButtonCornerRuntime::default, |rt| {
        if !rt.spring.is_initialized() {
            // Initialize lazily with the default radius to avoid an animated "pop" on first frame.
            rt.spring.reset(now_frame, base_radius);
        }

        rt.spring.set_target(now_frame, target, spring);
        rt.spring.advance(now_frame);
        (Corners::all(Px(rt.spring.value())), rt.spring.is_active())
    })
}

#[derive(Debug, Clone, Copy)]
struct ButtonSizeTokens {
    container_height: Px,
    leading_space: Px,
    trailing_space: Px,
}

fn button_size_tokens(theme: &Theme, size: ButtonSize) -> ButtonSizeTokens {
    match size {
        ButtonSize::Small => ButtonSizeTokens {
            container_height: theme
                .metric_by_key("md.comp.button.small.container.height")
                .unwrap_or(Px(40.0)),
            leading_space: theme
                .metric_by_key("md.comp.button.small.leading-space")
                .unwrap_or(Px(16.0)),
            trailing_space: theme
                .metric_by_key("md.comp.button.small.trailing-space")
                .unwrap_or(Px(16.0)),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct ButtonOutline {
    width: Px,
    color: Color,
}

fn button_outline(theme: &Theme, variant: ButtonVariant, enabled: bool) -> Option<ButtonOutline> {
    if variant != ButtonVariant::Outlined {
        return None;
    }
    let width = theme
        .metric_by_key("md.comp.button.small.outlined.outline.width")
        .unwrap_or(Px(1.0));

    let mut color = if enabled {
        theme
            .color_by_key("md.comp.button.outlined.outline.color")
            .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
            .or_else(|| theme.color_by_key("md.sys.color.outline"))
    } else {
        theme
            .color_by_key("md.comp.button.outlined.disabled.outline.color")
            .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
            .or_else(|| theme.color_by_key("md.sys.color.outline"))
    }
    .unwrap_or_else(|| theme.color_required("md.sys.color.outline"));

    color.a = 1.0;
    Some(ButtonOutline { width, color })
}
