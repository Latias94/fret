//! Material 3 button (MVP).
//!
//! This starts with outcome alignment:
//! - State layer (hover/pressed/focus) using `fret_ui::paint::paint_state_layer`.
//! - Bounded ripple using `fret_ui::paint::paint_ripple`.
//!
//! The state machines live in this crate; the renderer-facing primitives remain mechanism-only in
//! `crates/fret-ui`.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole, SvgFit, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_runtime::ActionId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::typography::{self, TextIntent};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::elevation::{
    AnimatedElevationRuntime, MaterialElevationInteraction, animate_material_elevation,
};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config_in_scope,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::motion_roles::{MaterialMotionRole, material_motion_spring_in_scope};
use crate::foundation::style_overrides::merge_style_override_slots;
use crate::foundation::surface::material_surface_style;
use crate::motion::{SpringAnimator, SpringSpec};
use crate::tokens::button::{self as button_tokens, ButtonSizeTokens};

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
    XSmall,
    #[default]
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Debug, Clone, Default)]
pub struct ButtonStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub container_elevation: OverrideSlot<Px>,
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

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
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

    pub fn merged(self, other: Self) -> Self {
        merge_style_override_slots!(
            self,
            other,
            [
                container_background,
                container_elevation,
                label_color,
                outline_color,
                state_layer_color,
            ]
        )
    }
}

#[derive(Clone)]
pub struct Button {
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    action: Option<ActionId>,
    on_activate: Option<OnActivate>,
    style: ButtonStyle,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("label", &self.label)
            .field("a11y_label", &self.a11y_label)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field(
                "leading_icon",
                &self.leading_icon.as_ref().map(|i| i.as_str()),
            )
            .field(
                "trailing_icon",
                &self.trailing_icon.as_ref().map(|i| i.as_str()),
            )
            .field("action", &self.action)
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
            a11y_label: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            leading_icon: None,
            trailing_icon: None,
            action: None,
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

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Bind a stable action ID to this button (action-first authoring).
    pub fn action(mut self, action: impl Into<ActionId>) -> Self {
        self.action = Some(action.into());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            control_chrome_pressable_with_id_props(cx, |cx, st, pressable_id| {
                let action_enabled = self
                    .action
                    .as_ref()
                    .is_none_or(|action| cx.command_is_enabled(action));
                let enabled = !self.disabled && action_enabled;

                if let Some(action) = self.action.clone() {
                    cx.pressable_dispatch_action_if_enabled(action);
                }
                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_add_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let pressed = enabled && st.pressed;
                let (base_radius, pressed_radius, corner_spring, size_tokens, label_style) = {
                    let theme = Theme::global(&*cx.app);
                    let base_radius = button_tokens::container_shape_radius(theme, self.size).0;
                    let pressed_radius =
                        button_tokens::pressed_container_shape_radius(theme, self.size).0;
                    let scheme_spring = material_motion_spring_in_scope(
                        &*cx,
                        theme,
                        MaterialMotionRole::ButtonPressedShape,
                    );
                    let corner_spring = button_pressed_corner_spring(scheme_spring);
                    let size_tokens = button_tokens::size_tokens(theme, self.size);
                    let label_style = button_label_style(theme, self.size);
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
                        label: Some(self.a11y_label.as_ref().unwrap_or(&self.label).clone()),
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
                    focus_ring_always_paint: false,
                    focus_ring_bounds: None,
                };

                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && st.hovered;
                let is_focused = enabled && st.focused && focus_visible;

                let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                let token_interaction = interaction.map(|s| match s {
                    PressableInteraction::Hovered => button_tokens::ButtonInteraction::Hovered,
                    PressableInteraction::Focused => button_tokens::ButtonInteraction::Focused,
                    PressableInteraction::Pressed => button_tokens::ButtonInteraction::Pressed,
                });
                let states = WidgetStates::from_pressable(cx, st, enabled);

                let (
                    label_color,
                    icon_color,
                    container_bg,
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    outline,
                    elevation_target,
                    shadow_color,
                ) = {
                    let theme = Theme::global(&*cx.app);

                    let label_color = resolve_override_slot_with(
                        self.style.label_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || button_tokens::label_color(theme, self.variant, enabled),
                    );
                    let icon_color = button_tokens::icon_color(
                        theme,
                        self.variant,
                        enabled,
                        label_color,
                        token_interaction,
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
                    let elevation_target = button_tokens::container_elevation(
                        theme,
                        self.variant,
                        enabled,
                        token_interaction,
                    );
                    let elevation_target = resolve_override_slot_with(
                        self.style.container_elevation.as_ref(),
                        states,
                        |elevation| *elevation,
                        || elevation_target,
                    );
                    let shadow_color = button_tokens::container_shadow_color(theme, self.variant);
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
                    let config = material_pressable_indication_config_in_scope(&*cx, None);

                    let outline = button_outline(theme, self.variant, enabled, size_tokens);
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
                        icon_color,
                        container_bg,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        outline,
                        elevation_target,
                        shadow_color,
                    )
                };

                let elevation_interaction =
                    token_interaction.map(|interaction| match interaction {
                        button_tokens::ButtonInteraction::Pressed => {
                            MaterialElevationInteraction::Pressed
                        }
                        button_tokens::ButtonInteraction::Hovered => {
                            MaterialElevationInteraction::Hovered
                        }
                        button_tokens::ButtonInteraction::Focused => {
                            MaterialElevationInteraction::Focused
                        }
                    });
                let (elevation, elevation_want_frames) =
                    cx.state_for(pressable_id, AnimatedElevationRuntime::default, |rt| {
                        animate_material_elevation(
                            rt,
                            now_frame,
                            enabled,
                            elevation_interaction,
                            elevation_target,
                        )
                    });

                let chrome = material_button_chrome_props(
                    cx,
                    container_bg,
                    corner_radii,
                    outline,
                    elevation,
                    shadow_color,
                    size_tokens,
                );

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    // PointerRegion is used to record `PointerRegionState.last_down` so ripple
                    // origins can align to pointer position without custom hook plumbing.
                    cx.pointer_region(props, move |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

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
                            corner_want_frames || elevation_want_frames,
                        );

                        let label =
                            material_button_label(cx, label_style, &self.label, label_color);
                        let has_icon = self.leading_icon.is_some() || self.trailing_icon.is_some();
                        let gap = if has_icon {
                            size_tokens.icon_label_space
                        } else {
                            Px(0.0)
                        };

                        let mut children = Vec::new();
                        if let Some(icon) = self.leading_icon.as_ref() {
                            children.push(material_button_icon(
                                cx,
                                icon,
                                size_tokens.icon_size,
                                icon_color,
                            ));
                        }
                        children.push(label);
                        if let Some(icon) = self.trailing_icon.as_ref() {
                            children.push(material_button_icon(
                                cx,
                                icon,
                                size_tokens.icon_size,
                                icon_color,
                            ));
                        }

                        let content = material_button_content(cx, size_tokens, gap, children);
                        vec![overlay, content]
                    })
                });

                (pressable_props, chrome, move |_cx| vec![pointer_region])
            })
        })
    }
}

fn material_button_chrome_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    background: Option<Color>,
    corner_radii: Corners,
    outline: Option<ButtonOutline>,
    elevation: Px,
    shadow_color: Color,
    size: ButtonSizeTokens,
) -> ContainerProps {
    let mut props = ContainerProps::default();
    if let Some(background) = background {
        let surface = {
            let theme = Theme::global(&*cx.app);
            material_surface_style(
                theme,
                background,
                elevation,
                Some(shadow_color),
                corner_radii,
            )
        };
        props.background = Some(surface.background);
        props.shadow = surface.shadow;
    }
    props.corner_radii = corner_radii;
    props.layout.size.min_width = Some(Length::Px(size.min_width));
    props.layout.size.min_height = Some(Length::Px(size.container_height));
    if let Some(outline) = outline {
        props.border = Edges::all(outline.width);
        props.border_color = Some(outline.color);
    }
    props
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
    props.layout.size.min_width = Some(Length::Px(Px(0.0)));
    props.layout.flex.shrink = 1.0;
    cx.text_props(props)
}

fn material_button_icon<H: UiHost>(
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

fn material_button_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: ButtonSizeTokens,
    gap: Px,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    // The outer container has no padding so the ink layer (absolute) can cover the full button.
    // Apply padding here instead.
    layout.size.height = Length::Px(size.container_height);
    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: gap.into(),
            padding: Edges {
                left: size.leading_space,
                right: size.trailing_space,
                top: Px(0.0),
                bottom: Px(0.0),
            }
            .into(),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| children,
    )
}

#[derive(Debug, Default, Clone)]
struct ButtonCornerRuntime {
    spring: SpringAnimator,
}

fn button_label_text_key(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::XSmall => "md.comp.button.xsmall.label-text",
        ButtonSize::Small => "md.comp.button.small.label-text",
        ButtonSize::Medium => "md.comp.button.medium.label-text",
        ButtonSize::Large => "md.comp.button.large.label-text",
        ButtonSize::XLarge => "md.comp.button.xlarge.label-text",
    }
}

fn button_label_style(theme: &Theme, size: ButtonSize) -> fret_core::TextStyle {
    let style = theme
        .text_style_by_key(button_label_text_key(size))
        .or_else(|| theme.text_style_by_key("md.comp.button.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.label-large"))
        .or_else(|| theme.text_style_by_key("text_style.button"))
        .unwrap_or_default();
    typography::with_intent(style, TextIntent::Control)
}

fn button_pressed_corner_spring(scheme: SpringSpec) -> SpringSpec {
    // Compose Material3 intentionally drives Button shape morphing with DefaultEffects to avoid
    // the spatial spring bounce used by larger moving surfaces.
    scheme
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

    cx.state_for(pressable_id, ButtonCornerRuntime::default, |rt| {
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
struct ButtonOutline {
    width: Px,
    color: Color,
}

fn button_outline(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    size_tokens: ButtonSizeTokens,
) -> Option<ButtonOutline> {
    if variant != ButtonVariant::Outlined {
        return None;
    }
    let width = size_tokens.outlined_outline_width;
    let color = button_tokens::outlined_outline_color(theme, enabled);
    Some(ButtonOutline { width, color })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{Point, Rect, Size};
    use fret_icons::ids;
    use fret_ui::element::{ElementKind, Length, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    #[test]
    fn button_labels_can_shrink_between_icon_slots() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let label = Arc::<str>::from(
            "A very long material button label that should shrink between the icon slots",
        );

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-button", |cx| {
                Button::new(label.clone())
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .into_element(cx)
            });

        let label = find_text_by_content(&el, label.as_ref()).expect("button label text");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.size.width, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.flex.shrink, 1.0);
    }

    #[test]
    fn button_chrome_applies_material_min_width() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-button", |cx| {
            let theme = Theme::global(&*cx.app);
            let tokens = button_tokens::size_tokens(theme, ButtonSize::Small);
            let props = material_button_chrome_props(
                cx,
                None,
                Corners::all(Px(20.0)),
                None,
                Px(0.0),
                Color::TRANSPARENT,
                tokens,
            );

            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(64.0))));
        });
    }

    #[test]
    fn button_elevation_tokens_follow_material_state_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(
            button_tokens::container_elevation(theme, ButtonVariant::Elevated, true, None),
            Px(1.0)
        );
        assert_eq!(
            button_tokens::container_elevation(
                theme,
                ButtonVariant::Elevated,
                true,
                Some(button_tokens::ButtonInteraction::Hovered),
            ),
            Px(3.0)
        );
        assert_eq!(
            button_tokens::container_elevation(
                theme,
                ButtonVariant::Filled,
                true,
                Some(button_tokens::ButtonInteraction::Hovered),
            ),
            Px(1.0)
        );
        assert_eq!(
            button_tokens::container_elevation(
                theme,
                ButtonVariant::Filled,
                true,
                Some(button_tokens::ButtonInteraction::Pressed),
            ),
            Px(0.0)
        );
    }

    #[test]
    fn outlined_button_outline_uses_typed_token_accessor() {
        let app = App::new();
        let theme = Theme::global(&app);
        let size_tokens = button_tokens::size_tokens(theme, ButtonSize::Small);

        let enabled = button_outline(theme, ButtonVariant::Outlined, true, size_tokens)
            .expect("outlined buttons produce an outline");
        let disabled = button_outline(theme, ButtonVariant::Outlined, false, size_tokens)
            .expect("outlined buttons produce a disabled outline");

        assert_eq!(
            enabled.color,
            button_tokens::outlined_outline_color(theme, true)
        );
        assert_eq!(
            disabled.color,
            button_tokens::outlined_outline_color(theme, false)
        );
    }

    #[test]
    fn button_pressed_shape_motion_uses_default_effects_spring() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let cfg =
            crate::tokens::v30::theme_config(crate::tokens::v30::TypographyOptions::default());
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&cfg);
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-button", |cx| {
            let theme = Theme::global(&*cx.app);
            let expected =
                material_motion_spring_in_scope(cx, theme, MaterialMotionRole::ButtonPressedShape);
            let actual = button_pressed_corner_spring(expected);

            assert_eq!(actual, expected);
            assert!(
                actual.damping >= 1.0,
                "DefaultEffects should stay non-bouncy for button pressed shape motion"
            );
        });
    }
}
