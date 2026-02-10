//! Material 3 carousel item (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven surface + outline via `md.comp.carousel-item.*`.
//! - State layer + bounded ripple using the shared Material foundation indication path.

use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, OpacityProps, Overflow, PointerRegionProps, PressableA11y,
    PressableProps,
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
use crate::foundation::interaction::pressable_interaction;
use crate::foundation::surface::material_surface_style;
use crate::tokens::carousel_item as carousel_item_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselItemVariant {
    #[default]
    Standard,
    WithOutline,
}

#[derive(Debug, Clone, Default)]
pub struct CarouselItemStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl CarouselItemStyle {
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
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct CarouselItem {
    variant: CarouselItemVariant,
    on_activate: Option<OnActivate>,
    style: CarouselItemStyle,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    width: Option<Px>,
    height: Option<Px>,
}

impl std::fmt::Debug for CarouselItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CarouselItem")
            .field("variant", &self.variant)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl CarouselItem {
    pub fn new() -> Self {
        Self {
            variant: CarouselItemVariant::default(),
            on_activate: None,
            style: CarouselItemStyle::default(),
            disabled: false,
            a11y_label: None,
            test_id: None,
            width: None,
            height: None,
        }
    }

    pub fn variant(mut self, variant: CarouselItemVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: CarouselItemStyle) -> Self {
        self.style = self.style.merged(style);
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

    pub fn width(mut self, width: Px) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Px) -> Self {
        self.height = Some(height);
        self
    }

    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        cx.scope(|cx| {
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let interactive = self.on_activate.is_some();
                let enabled = interactive && !self.disabled;

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let (corner_radii, focus_ring) = {
                    let theme = Theme::global(&*cx.app);
                    let corner_radii = carousel_item_tokens::container_shape(theme);
                    let focus_ring = enabled.then(|| {
                        material_focus_ring_for_component(
                            theme,
                            carousel_item_tokens::COMPONENT_PREFIX,
                            corner_radii,
                        )
                    });
                    (corner_radii, focus_ring)
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    key_activation: Default::default(),
                    a11y: PressableA11y {
                        role: interactive.then_some(SemanticsRole::Button),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l
                    },
                    focus_ring,
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
                        let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);

                        let states = WidgetStates::from_pressable(cx, st, enabled);

                        let (
                            surface,
                            outline,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            disabled_opacity,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let container_bg =
                                carousel_item_tokens::container_background(theme, self.disabled);
                            let container_bg = resolve_override_slot_with(
                                self.style.container_background.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || container_bg,
                            );

                            let elevation = carousel_item_tokens::container_elevation(
                                theme,
                                self.disabled,
                                interaction,
                            );
                            let shadow_color = carousel_item_tokens::container_shadow_color(theme);
                            let surface = material_surface_style(
                                theme,
                                container_bg,
                                elevation,
                                Some(shadow_color),
                                corner_radii,
                            );

                            let outline = carousel_item_tokens::outline(
                                theme,
                                self.variant == CarouselItemVariant::WithOutline,
                                self.disabled,
                                interaction,
                            );
                            let outline = resolve_override_slot_opt_with(
                                self.style.outline_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || outline.map(|o| o.color),
                            )
                            .map(|color| {
                                carousel_item_tokens::CarouselItemOutline {
                                    color,
                                    width: outline.map(|o| o.width).unwrap_or(Px(0.0)),
                                }
                            });

                            let state_layer_color =
                                carousel_item_tokens::state_layer_color(theme, interaction);
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let state_layer_target =
                                carousel_item_tokens::state_layer_opacity(theme, interaction);
                            let ripple_base_opacity =
                                carousel_item_tokens::pressed_state_layer_opacity(theme);
                            let config = material_pressable_indication_config(theme, None);

                            let disabled_opacity = self
                                .disabled
                                .then(|| carousel_item_tokens::disabled_opacity(theme));

                            (
                                surface,
                                outline,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                config,
                                disabled_opacity,
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

                        let mut container = ContainerProps::default();
                        container.background = Some(surface.background);
                        container.shadow = surface.shadow;
                        container.corner_radii = corner_radii;
                        container.layout.overflow = Overflow::Clip;
                        container.layout.size.width =
                            self.width.map(Length::Px).unwrap_or(Length::Fill);
                        container.layout.size.height =
                            self.height.map(Length::Px).unwrap_or(Length::Auto);
                        if let Some(outline) = outline {
                            container.border = Edges::all(outline.width);
                            container.border_color = Some(outline.color);
                        }

                        let children: Vec<AnyElement> =
                            std::iter::once(overlay).chain(content(cx)).collect();
                        let container = cx.container(container, move |_cx| children);

                        if let Some(opacity) = disabled_opacity {
                            let mut props = OpacityProps::default();
                            props.opacity = opacity;
                            vec![cx.opacity_props(props, move |_cx| vec![container])]
                        } else {
                            vec![container]
                        }
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}
