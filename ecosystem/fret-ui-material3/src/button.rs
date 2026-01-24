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

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    IndicationConfig, advance_indication_for_pressable, material_ink_layer,
};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::{SpringAnimator, SpringSpec};

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

#[derive(Clone)]
pub struct Button {
    label: Arc<str>,
    variant: ButtonVariant,
    size: ButtonSize,
    on_activate: Option<OnActivate>,
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
            let theme = Theme::global(&*cx.app).clone();

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let pressed = enabled && st.pressed;
                let (corner_radii, corner_want_frames) =
                    animated_button_corner_radii(cx, &theme, pressable_id, now_frame, pressed);
                let size_tokens = button_size_tokens(&theme, self.size);

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
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
                    focus_ring: Some(material_focus_ring_for_component(
                        &theme,
                        "md.comp.button",
                        corner_radii,
                    )),
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

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);
                        let (container_bg, label_color) =
                            button_colors(&theme, self.variant, enabled);
                        let state_layer_color =
                            state_layer_color(&theme, self.variant, label_color, interaction);

                        let state_layer_target = state_layer_target_opacity(
                            &theme,
                            enabled,
                            self.variant,
                            is_pressed,
                            is_hovered,
                            is_focused,
                        );

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
                        let ripple_base_opacity = theme
                            .number_by_key(&format!(
                                "md.comp.button.{}.pressed.state-layer.opacity",
                                variant_key(self.variant)
                            ))
                            .unwrap_or(0.1);
                        let config = IndicationConfig {
                            state_duration_ms,
                            ripple_expand_ms,
                            ripple_fade_ms,
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
                            state_layer_color,
                            indication.state_layer_opacity,
                            indication.ripple_frame,
                            indication.want_frames || corner_want_frames,
                        );

                        let label = material_button_label(cx, &theme, &self.label, label_color);
                        let content = material_button_content(cx, size_tokens, label);

                        let outline = button_outline(&theme, self.variant, enabled);
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
    theme: &Theme,
    label: &Arc<str>,
    color: Color,
) -> AnyElement {
    let style = theme
        .text_style_by_key("md.sys.typescale.label-large")
        .or_else(|| theme.text_style_by_key("text_style.button"))
        .unwrap_or_else(|| fret_core::TextStyle::default());

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
    theme: &Theme,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    pressed: bool,
) -> (Corners, bool) {
    let base = button_shape_radius(theme);
    let pressed_radius = button_pressed_shape_radius(theme);
    let target = if pressed { pressed_radius } else { base };

    let scheme_spring = sys_spring_in_scope(cx, theme, MotionSchemeKey::FastSpatial);
    let spring = button_pressed_corner_spring(theme, scheme_spring);

    cx.with_state_for(pressable_id, ButtonCornerRuntime::default, |rt| {
        if !rt.spring.is_initialized() {
            // Initialize lazily with the default radius to avoid an animated "pop" on first frame.
            rt.spring.reset(now_frame, base);
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

fn button_colors(theme: &Theme, variant: ButtonVariant, enabled: bool) -> (Option<Color>, Color) {
    let variant_key = variant_key(variant);
    let disabled_label_opacity = theme
        .number_by_key(&format!(
            "md.comp.button.{variant_key}.disabled.label-text.opacity"
        ))
        .unwrap_or(0.38);
    let disabled_container_opacity = theme
        .number_by_key(&format!(
            "md.comp.button.{variant_key}.disabled.container.opacity"
        ))
        .unwrap_or(0.1);

    let disabled_label_base = theme
        .color_by_key(&format!(
            "md.comp.button.{variant_key}.disabled.label-text.color"
        ))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    let mut disabled_label = disabled_label_base;
    disabled_label.a *= disabled_label_opacity;

    let enabled_label = theme
        .color_by_key(&format!("md.comp.button.{variant_key}.label-text.color"))
        .or_else(|| match variant {
            ButtonVariant::Filled => theme.color_by_key("md.sys.color.on-primary"),
            ButtonVariant::Tonal => theme.color_by_key("md.sys.color.on-secondary-container"),
            ButtonVariant::Elevated | ButtonVariant::Text => {
                theme.color_by_key("md.sys.color.primary")
            }
            ButtonVariant::Outlined => theme.color_by_key("md.sys.color.on-surface-variant"),
        })
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    let label = if enabled {
        enabled_label
    } else {
        disabled_label
    };

    let background = match variant {
        ButtonVariant::Text | ButtonVariant::Outlined => None,
        ButtonVariant::Filled => {
            if enabled {
                Some(
                    theme
                        .color_by_key("md.comp.button.filled.container.color")
                        .or_else(|| theme.color_by_key("md.sys.color.primary"))
                        .unwrap_or_else(|| theme.color_required("md.sys.color.primary")),
                )
            } else {
                let mut c = theme
                    .color_by_key("md.comp.button.filled.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
                c.a *= disabled_container_opacity;
                Some(c)
            }
        }
        ButtonVariant::Tonal => {
            if enabled {
                Some(
                    theme
                        .color_by_key("md.comp.button.tonal.container.color")
                        .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
                        .unwrap_or_else(|| {
                            theme.color_required("md.sys.color.secondary-container")
                        }),
                )
            } else {
                let mut c = theme
                    .color_by_key("md.comp.button.tonal.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
                c.a *= disabled_container_opacity;
                Some(c)
            }
        }
        ButtonVariant::Elevated => {
            if enabled {
                Some(
                    theme
                        .color_by_key("md.comp.button.elevated.container.color")
                        .or_else(|| theme.color_by_key("md.sys.color.surface-container-low"))
                        .unwrap_or_else(|| {
                            theme.color_required("md.sys.color.surface-container-low")
                        }),
                )
            } else {
                let mut c = theme
                    .color_by_key("md.comp.button.elevated.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
                c.a *= disabled_container_opacity;
                Some(c)
            }
        }
    };

    (background, label)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionState {
    Hovered,
    Focused,
    Pressed,
}

fn interaction_state(pressed: bool, hovered: bool, focused: bool) -> Option<InteractionState> {
    if pressed {
        return Some(InteractionState::Pressed);
    }
    if focused {
        return Some(InteractionState::Focused);
    }
    if hovered {
        return Some(InteractionState::Hovered);
    }
    None
}

fn state_layer_color(
    theme: &Theme,
    variant: ButtonVariant,
    label: Color,
    state: Option<InteractionState>,
) -> Color {
    let Some(state) = state else {
        return label;
    };
    let variant_key = variant_key(variant);
    let suffix = match state {
        InteractionState::Hovered => "hovered.state-layer.color",
        InteractionState::Focused => "focused.state-layer.color",
        InteractionState::Pressed => "pressed.state-layer.color",
    };

    theme
        .color_by_key(&format!("md.comp.button.{variant_key}.{suffix}"))
        .unwrap_or(label)
}

fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    variant: ButtonVariant,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> f32 {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        return 0.0;
    }
    let variant_key = variant_key(variant);
    if pressed {
        return tokens.number_comp_or_sys(
            &format!("md.comp.button.{variant_key}.pressed.state-layer.opacity"),
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        );
    }
    if focused {
        return tokens.number_comp_or_sys(
            &format!("md.comp.button.{variant_key}.focused.state-layer.opacity"),
            "md.sys.state.focus.state-layer-opacity",
            0.1,
        );
    }
    if hovered {
        return tokens.number_comp_or_sys(
            &format!("md.comp.button.{variant_key}.hovered.state-layer.opacity"),
            "md.sys.state.hover.state-layer-opacity",
            0.08,
        );
    }
    0.0
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

fn variant_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "filled",
        ButtonVariant::Tonal => "tonal",
        ButtonVariant::Elevated => "elevated",
        ButtonVariant::Outlined => "outlined",
        ButtonVariant::Text => "text",
    }
}
