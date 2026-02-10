//! Material 3 floating action button (FAB) (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven variants/sizing via `md.comp.fab.*` / `md.comp.extended-fab.*`.
//! - Bounded ripple + state layer + focus-visible ring.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole, SvgFit, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_with,
};

use crate::foundation::arc_str::empty_arc_str;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::surface::material_surface_style;
use crate::tokens::fab as fab_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FabVariant {
    #[default]
    Surface,
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FabSize {
    #[default]
    Medium,
    Small,
    Large,
}

#[derive(Debug, Clone, Default)]
pub struct FabStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub icon_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl FabStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
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
        if other.icon_color.is_some() {
            self.icon_color = other.icon_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Fab {
    icon: Option<IconId>,
    label: Option<Arc<str>>,
    variant: FabVariant,
    size: FabSize,
    lowered: bool,
    on_activate: Option<OnActivate>,
    style: FabStyle,
    disabled: bool,
    test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
}

impl std::fmt::Debug for Fab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fab")
            .field("icon", &self.icon.as_ref().map(|i| i.as_str()))
            .field("label", &self.label)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("lowered", &self.lowered)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Fab {
    pub fn new(icon: IconId) -> Self {
        Self {
            icon: Some(icon),
            label: None,
            variant: FabVariant::default(),
            size: FabSize::default(),
            lowered: false,
            on_activate: None,
            style: FabStyle::default(),
            disabled: false,
            test_id: None,
            a11y_label: None,
        }
    }

    /// Makes this an "extended" FAB by providing a label.
    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Removes the icon (extended FABs may omit their icon).
    pub fn icon(mut self, icon: Option<IconId>) -> Self {
        self.icon = icon;
        self
    }

    pub fn variant(mut self, variant: FabVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    pub fn lowered(mut self, lowered: bool) -> Self {
        self.lowered = lowered;
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

    pub fn style(mut self, style: FabStyle) -> Self {
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
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                let extended = self.label.is_some();
                let states = WidgetStates::from_pressable(cx, st, enabled);

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && st.hovered;
                let is_focused = enabled && st.focused && focus_visible;

                let tokens_interaction = pressable_interaction(is_pressed, is_hovered, is_focused)
                    .map(|s| match s {
                        PressableInteraction::Hovered => fab_tokens::FabInteraction::Hovered,
                        PressableInteraction::Focused => fab_tokens::FabInteraction::Focused,
                        PressableInteraction::Pressed => fab_tokens::FabInteraction::Pressed,
                    });

                let (corner_radii, layout, focus_ring) = {
                    let theme = Theme::global(&*cx.app);

                    let corner_radii = if extended {
                        fab_tokens::extended_container_shape(theme)
                    } else {
                        fab_tokens::container_shape(theme, self.size)
                    };

                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.overflow = Overflow::Visible;
                    enforce_minimum_interactive_size(&mut layout, theme);

                    let focus_ring = material_focus_ring_for_component(
                        theme,
                        focus_ring_component_prefix(extended, self.variant),
                        corner_radii,
                    );

                    (corner_radii, layout, focus_ring)
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: self.a11y_label.clone().or_else(|| self.label.clone()),
                        test_id: self.test_id.clone(),
                        ..Default::default()
                    },
                    layout,
                    focus_ring: Some(focus_ring),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    cx.pointer_region(props, |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let interaction = tokens_interaction;

                        let now_frame = cx.frame_id.0;
                        let (
                            surface,
                            icon_color,
                            label_color,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            content_tokens,
                            chrome_tokens,
                            icon_size,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let background = fab_tokens::container_background(
                                theme,
                                extended,
                                self.variant,
                                enabled,
                                self.lowered,
                            );
                            let background = resolve_override_slot_with(
                                self.style.container_background.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || background,
                            );

                            let icon_color = fab_tokens::icon_color(
                                theme,
                                extended,
                                self.variant,
                                enabled,
                                interaction,
                            );
                            let icon_color = resolve_override_slot_with(
                                self.style.icon_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || icon_color,
                            );

                            let label_color =
                                fab_tokens::label_color(theme, self.variant, enabled, interaction);
                            let label_color = resolve_override_slot_with(
                                self.style.label_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || label_color,
                            );

                            let state_layer_target = state_layer_target_opacity(
                                theme,
                                extended,
                                self.variant,
                                enabled,
                                is_pressed,
                                is_hovered,
                                is_focused,
                            );

                            let state_layer_color = tokens_interaction
                                .map(|s| {
                                    fab_tokens::state_layer_color(theme, extended, self.variant, s)
                                })
                                .unwrap_or_else(|| {
                                    fab_tokens::state_layer_color(
                                        theme,
                                        extended,
                                        self.variant,
                                        fab_tokens::FabInteraction::Pressed,
                                    )
                                });
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let elevation = fab_tokens::container_elevation(
                                theme,
                                extended,
                                self.variant,
                                enabled,
                                self.lowered,
                                interaction,
                            );
                            let shadow_color =
                                fab_tokens::container_shadow_color(theme, extended, self.variant);
                            let surface = material_surface_style(
                                theme,
                                background,
                                elevation,
                                Some(shadow_color),
                                corner_radii,
                            );

                            let ripple_base_opacity =
                                fab_tokens::pressed_state_layer_opacity_for_variant(
                                    theme,
                                    extended,
                                    self.variant,
                                );
                            let config = material_pressable_indication_config(theme, None);

                            let content_tokens = ExtendedFabContentTokens {
                                icon_size: fab_tokens::extended_icon_size(theme),
                                icon_label_space: fab_tokens::extended_icon_label_space(theme),
                                label_style: theme
                                    .text_style_by_key("md.comp.extended-fab.label-text")
                                    .or_else(|| {
                                        theme.text_style_by_key("md.sys.typescale.label-large")
                                    })
                                    .unwrap_or_else(|| fret_core::TextStyle::default()),
                            };

                            let chrome_tokens = ExtendedFabChromeTokens {
                                height: fab_tokens::extended_container_height(theme),
                                pad_left: fab_tokens::extended_leading_space(
                                    theme,
                                    self.icon.is_some(),
                                ),
                                pad_right: fab_tokens::extended_trailing_space(theme),
                            };

                            let icon_size = fab_tokens::icon_size(theme, self.size);

                            (
                                surface,
                                icon_color,
                                label_color,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                config,
                                content_tokens,
                                chrome_tokens,
                                icon_size,
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
                            false,
                        );

                        let content = if extended {
                            material_extended_fab_content(
                                cx,
                                content_tokens,
                                self.icon.as_ref(),
                                self.label.as_ref(),
                                icon_color,
                                label_color,
                            )
                        } else {
                            material_fab_content(cx, self.icon.as_ref(), icon_size, icon_color)
                        };

                        let chrome = if extended {
                            material_extended_fab_chrome(
                                cx,
                                chrome_tokens,
                                corner_radii,
                                surface.background,
                                surface.shadow,
                                overlay,
                                content,
                            )
                        } else {
                            material_fab_chrome(
                                cx,
                                fab_tokens::container_size(Theme::global(&*cx.app), self.size),
                                corner_radii,
                                surface.background,
                                surface.shadow,
                                overlay,
                                content,
                            )
                        };

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn state_layer_target_opacity(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let interaction = match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => fab_tokens::FabInteraction::Pressed,
        Some(PressableInteraction::Focused) => fab_tokens::FabInteraction::Focused,
        Some(PressableInteraction::Hovered) => fab_tokens::FabInteraction::Hovered,
        None => return 0.0,
    };

    fab_tokens::state_layer_opacity(theme, extended, variant, interaction)
}

fn material_fab_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container: Px,
    corner_radii: Corners,
    background: Color,
    shadow: Option<fret_ui::element::ShadowStyle>,
    overlay: AnyElement,
    content: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.layout.size.min_width = Some(container);
    props.layout.size.min_height = Some(container);
    props.background = Some(background);
    props.shadow = shadow;
    props.corner_radii = corner_radii;
    cx.container(props, move |_cx| vec![overlay, content])
}

#[derive(Debug, Clone, Copy)]
struct ExtendedFabChromeTokens {
    height: Px,
    pad_left: Px,
    pad_right: Px,
}

#[derive(Debug, Clone)]
struct ExtendedFabContentTokens {
    icon_size: Px,
    icon_label_space: Px,
    label_style: fret_core::TextStyle,
}

fn material_extended_fab_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tokens: ExtendedFabChromeTokens,
    corner_radii: Corners,
    background: Color,
    shadow: Option<fret_ui::element::ShadowStyle>,
    overlay: AnyElement,
    content: AnyElement,
) -> AnyElement {
    let height = tokens.height;
    let pad_left = tokens.pad_left;
    let pad_right = tokens.pad_right;

    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.layout.size.height = Length::Px(height);
    props.background = Some(background);
    props.shadow = shadow;
    props.corner_radii = corner_radii;

    let padded_content = {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.height = Length::Px(height);

        let mut props = FlexProps::default();
        props.layout = layout;
        props.direction = Axis::Horizontal;
        props.gap = Px(0.0);
        props.padding = Edges {
            left: pad_left,
            right: pad_right,
            top: Px(0.0),
            bottom: Px(0.0),
        };
        props.justify = MainAlign::Center;
        props.align = CrossAlign::Center;
        props.wrap = false;
        cx.flex(props, move |_cx| vec![content])
    };

    cx.container(props, move |_cx| vec![overlay, padded_content])
}

fn material_fab_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: Option<&IconId>,
    icon_size: Px,
    icon_color: Color,
) -> AnyElement {
    let child = if let Some(icon) = icon {
        material_icon(cx, icon, icon_size, icon_color)
    } else {
        let mut props = TextProps::new(empty_arc_str());
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;
        cx.text_props(props)
    };

    let mut props = FlexProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.direction = Axis::Horizontal;
    props.gap = Px(0.0);
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props.wrap = false;
    cx.flex(props, move |_cx| vec![child])
}

fn material_extended_fab_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tokens: ExtendedFabContentTokens,
    icon: Option<&IconId>,
    label: Option<&Arc<str>>,
    icon_color: Color,
    label_color: Color,
) -> AnyElement {
    let icon_size = tokens.icon_size;
    let icon_label_space = tokens.icon_label_space;

    let mut children: Vec<AnyElement> = Vec::new();
    if let Some(icon) = icon {
        children.push(material_icon(cx, icon, icon_size, icon_color));
        children.push({
            let mut spacer = fret_ui::element::SpacerProps::default();
            spacer.layout.size.width = Length::Px(icon_label_space);
            cx.spacer(spacer)
        });
    }

    if let Some(label) = label {
        children.push(material_extended_fab_label(
            cx,
            tokens.label_style,
            label,
            label_color,
        ));
    }

    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.gap = Px(0.0);
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props.wrap = false;
    cx.flex(props, move |_cx| children)
}

fn material_extended_fab_label<H: UiHost>(
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

fn focus_ring_component_prefix(extended: bool, variant: FabVariant) -> &'static str {
    match (extended, variant) {
        (false, FabVariant::Surface) => "md.comp.fab.surface",
        (false, FabVariant::Primary) => "md.comp.fab.primary-container",
        (false, FabVariant::Secondary) => "md.comp.fab.secondary-container",
        (false, FabVariant::Tertiary) => "md.comp.fab.tertiary-container",
        (true, FabVariant::Surface) => "md.comp.extended-fab.surface",
        (true, FabVariant::Primary) => "md.comp.extended-fab.primary-container",
        (true, FabVariant::Secondary) => "md.comp.extended-fab.secondary-container",
        (true, FabVariant::Tertiary) => "md.comp.extended-fab.tertiary-container",
    }
}
