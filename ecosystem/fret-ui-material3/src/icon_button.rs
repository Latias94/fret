//! Material 3 icon button (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven colors/sizing via `md.comp.icon-button.*`.
//! - State layer (hover/pressed/focus) + bounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole, SvgFit};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{SvgSource, Theme, UiHost};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::{SpringAnimator, SpringSpec};
use crate::tokens::icon_button as icon_button_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonVariant {
    #[default]
    Standard,
    Filled,
    Tonal,
    Outlined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonSize {
    #[default]
    Small,
}

#[derive(Clone)]
pub struct IconButton {
    icon: IconId,
    variant: IconButtonVariant,
    size: IconButtonSize,
    toggle: bool,
    selected: bool,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
}

impl std::fmt::Debug for IconButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IconButton")
            .field("icon", &self.icon.as_str())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("selected", &self.selected)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl IconButton {
    pub fn new(icon: IconId) -> Self {
        Self {
            icon,
            variant: IconButtonVariant::default(),
            size: IconButtonSize::default(),
            toggle: false,
            selected: false,
            on_activate: None,
            disabled: false,
            test_id: None,
            a11y_label: None,
        }
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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

                let size_tokens = icon_button_size_tokens(&theme, self.size);
                let now_frame = cx.frame_id.0;
                let pressed = enabled && st.pressed;
                let (corner_radii, corner_want_frames) =
                    animated_icon_button_corner_radii(cx, &theme, pressable_id, now_frame, pressed);

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        selected: self.toggle && self.selected,
                        checked: self.toggle.then_some(self.selected),
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
                        "md.comp.icon-button",
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

                        let focus_visible =
                            fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                        let is_pressed = enabled && st.pressed;
                        let is_hovered = enabled && st.hovered;
                        let is_focused = enabled && st.focused && focus_visible;

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                        let colors = icon_button_colors(
                            &theme,
                            self.variant,
                            size_tokens.outline_width,
                            self.toggle,
                            self.selected,
                            enabled,
                            interaction,
                        );

                        let state_layer_target = state_layer_target_opacity(
                            &theme,
                            self.variant,
                            enabled,
                            is_pressed,
                            is_hovered,
                            is_focused,
                        );

                        let ripple_base_opacity =
                            icon_button_tokens::pressed_state_layer_opacity(&theme, self.variant);
                        let config = material_pressable_indication_config(&theme, None);
                        let overlay = material_ink_layer_for_pressable(
                            cx,
                            pressable_id,
                            now_frame,
                            corner_radii,
                            RippleClip::Bounded,
                            colors.state_layer_color,
                            is_pressed,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            corner_want_frames,
                        );

                        let icon =
                            material_icon(cx, &self.icon, size_tokens.icon_size, colors.icon_color);
                        let content = material_icon_button_content(cx, size_tokens, icon);
                        let chrome = material_icon_button_chrome(
                            cx,
                            size_tokens,
                            corner_radii,
                            colors.background,
                            colors.outline,
                            vec![overlay, content],
                        );

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

#[derive(Debug, Default, Clone)]
struct IconButtonCornerRuntime {
    spring: SpringAnimator,
}

fn icon_button_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.icon-button.container.shape.round")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
        .0
}

fn icon_button_pressed_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.icon-button.pressed.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Px(8.0))
        .0
}

fn icon_button_pressed_corner_spring(theme: &Theme, scheme_fallback: SpringSpec) -> SpringSpec {
    let tokens = MaterialTokenResolver::new(theme);
    SpringSpec {
        damping: tokens.number_comp_or_sys(
            "md.comp.icon-button.pressed.container.corner-size.motion.spring.damping",
            "md.sys.motion.spring.fast.spatial.damping",
            scheme_fallback.damping,
        ),
        stiffness: tokens.number_comp_or_sys(
            "md.comp.icon-button.pressed.container.corner-size.motion.spring.stiffness",
            "md.sys.motion.spring.fast.spatial.stiffness",
            scheme_fallback.stiffness,
        ),
    }
}

fn animated_icon_button_corner_radii<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    pressable_id: fret_ui::elements::GlobalElementId,
    now_frame: u64,
    pressed: bool,
) -> (Corners, bool) {
    let base = icon_button_shape_radius(theme);
    let pressed_radius = icon_button_pressed_shape_radius(theme);
    let target = if pressed { pressed_radius } else { base };

    let scheme_spring = sys_spring_in_scope(cx, theme, MotionSchemeKey::FastSpatial);
    let spring = icon_button_pressed_corner_spring(theme, scheme_spring);

    cx.with_state_for(pressable_id, IconButtonCornerRuntime::default, |rt| {
        if !rt.spring.is_initialized() {
            rt.spring.reset(now_frame, base);
        }

        rt.spring.set_target(now_frame, target, spring);
        rt.spring.advance(now_frame);
        (Corners::all(Px(rt.spring.value())), rt.spring.is_active())
    })
}

#[derive(Debug, Clone, Copy)]
struct IconButtonSizeTokens {
    container: Px,
    pad_left: Px,
    pad_right: Px,
    icon_size: Px,
    outline_width: Px,
}

fn icon_button_size_tokens(theme: &Theme, size: IconButtonSize) -> IconButtonSizeTokens {
    match size {
        IconButtonSize::Small => IconButtonSizeTokens {
            container: theme
                .metric_by_key("md.comp.icon-button.small.container.height")
                .unwrap_or(Px(40.0)),
            pad_left: theme
                .metric_by_key("md.comp.icon-button.small.default.leading-space")
                .unwrap_or(Px(8.0)),
            pad_right: theme
                .metric_by_key("md.comp.icon-button.small.default.trailing-space")
                .unwrap_or(Px(8.0)),
            icon_size: theme
                .metric_by_key("md.comp.icon-button.small.icon.size")
                .unwrap_or(Px(24.0)),
            outline_width: theme
                .metric_by_key("md.comp.icon-button.small.outlined.outline.width")
                .unwrap_or(Px(1.0)),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct IconButtonColors {
    background: Option<Color>,
    icon_color: Color,
    state_layer_color: Color,
    outline: Option<IconButtonOutline>,
}

#[derive(Debug, Clone, Copy)]
struct IconButtonOutline {
    width: Px,
    color: Color,
}

fn icon_button_colors(
    theme: &Theme,
    variant: IconButtonVariant,
    outline_width: Px,
    toggle: bool,
    selected: bool,
    enabled: bool,
    interaction: Option<InteractionState>,
) -> IconButtonColors {
    let tokens_interaction = interaction.map(|s| match s {
        InteractionState::Hovered => icon_button_tokens::IconButtonInteraction::Hovered,
        InteractionState::Focused => icon_button_tokens::IconButtonInteraction::Focused,
        InteractionState::Pressed => icon_button_tokens::IconButtonInteraction::Pressed,
    });

    let icon_color = icon_button_tokens::icon_color(
        theme,
        variant,
        toggle,
        selected,
        enabled,
        tokens_interaction,
    );
    let state_layer_color = icon_button_tokens::state_layer_color(
        theme,
        variant,
        toggle,
        selected,
        enabled,
        tokens_interaction,
    );
    let background = icon_button_tokens::container_background(
        theme, variant, toggle, selected, enabled, icon_color,
    );

    let outline = (variant == IconButtonVariant::Outlined).then(|| IconButtonOutline {
        width: outline_width,
        color: icon_button_tokens::outlined_outline_color(theme, enabled),
    });

    IconButtonColors {
        background,
        icon_color,
        state_layer_color,
        outline,
    }
}

fn material_icon_button_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: IconButtonSizeTokens,
    corner_radii: Corners,
    background: Option<Color>,
    outline: Option<IconButtonOutline>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.layout.size.min_width = Some(size.container);
    props.layout.size.min_height = Some(size.container);
    props.background = background;
    props.corner_radii = corner_radii;
    if let Some(outline) = outline {
        props.border = Edges::all(outline.width);
        props.border_color = Some(outline.color);
    }
    cx.container(props, move |_cx| children)
}

fn material_icon_button_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: IconButtonSizeTokens,
    icon: AnyElement,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = Length::Px(size.container);
    layout.size.height = Length::Px(size.container);
    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges {
                left: size.pad_left,
                right: size.pad_right,
                top: Px(0.0),
                bottom: Px(0.0),
            },
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![icon],
    )
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

fn state_layer_target_opacity(
    theme: &Theme,
    variant: IconButtonVariant,
    enabled: bool,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    if pressed {
        return icon_button_tokens::state_layer_opacity(
            theme,
            variant,
            icon_button_tokens::IconButtonInteraction::Pressed,
        );
    }
    if focused {
        return icon_button_tokens::state_layer_opacity(
            theme,
            variant,
            icon_button_tokens::IconButtonInteraction::Focused,
        );
    }
    if hovered {
        return icon_button_tokens::state_layer_opacity(
            theme,
            variant,
            icon_button_tokens::IconButtonInteraction::Hovered,
        );
    }
    0.0
}
