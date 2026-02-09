//! Material 3 card (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven surface + outline via `md.comp.{filled,elevated,outlined}-card.*`.
//! - State layer + bounded ripple using the shared Material foundation indication path.

use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, Overflow, PointerRegionProps, PressableA11y, PressableProps,
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
use crate::tokens::card as card_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    #[default]
    Filled,
    Elevated,
    Outlined,
}

#[derive(Debug, Clone, Default)]
pub struct CardStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl CardStyle {
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
pub struct Card {
    variant: CardVariant,
    on_activate: Option<OnActivate>,
    style: CardStyle,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("variant", &self.variant)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Card {
    pub fn new() -> Self {
        Self {
            variant: CardVariant::default(),
            on_activate: None,
            style: CardStyle::default(),
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: CardStyle) -> Self {
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
                    let corner_radii = card_tokens::container_shape(theme, self.variant);
                    let focus_ring = enabled.then(|| {
                        material_focus_ring_for_component(
                            theme,
                            card_tokens::component_prefix(self.variant),
                            corner_radii,
                        )
                    });
                    (corner_radii, focus_ring)
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
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
                            container,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let container_bg = card_tokens::container_background(
                                theme,
                                self.variant,
                                !self.disabled,
                            );
                            let container_bg = resolve_override_slot_with(
                                self.style.container_background.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || container_bg,
                            );

                            let elevation = card_tokens::container_elevation(
                                theme,
                                self.variant,
                                !self.disabled,
                                interaction,
                            );
                            let shadow_color =
                                card_tokens::container_shadow_color(theme, self.variant);
                            let surface = material_surface_style(
                                theme,
                                container_bg,
                                elevation,
                                Some(shadow_color),
                                corner_radii,
                            );

                            let outline = card_tokens::outline(
                                theme,
                                self.variant,
                                !self.disabled,
                                interaction,
                            );
                            let outline = resolve_override_slot_opt_with(
                                self.style.outline_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || outline.map(|o| o.color),
                            )
                            .map(|color| card_tokens::CardOutline {
                                color,
                                width: outline.map(|o| o.width).unwrap_or(Px(0.0)),
                            });

                            let state_layer_color =
                                card_tokens::state_layer_color(theme, self.variant, interaction);
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let state_layer_target =
                                card_tokens::state_layer_opacity(theme, self.variant, interaction);
                            let ripple_base_opacity =
                                card_tokens::pressed_state_layer_opacity(theme, self.variant);
                            let config = material_pressable_indication_config(theme, None);

                            let mut container = ContainerProps::default();
                            container.background = Some(surface.background);
                            container.shadow = surface.shadow;
                            container.corner_radii = corner_radii;
                            container.layout.overflow = Overflow::Clip;
                            if let Some(outline) = outline {
                                container.border = Edges::all(outline.width);
                                container.border_color = Some(outline.color);
                            }

                            (
                                container,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                config,
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

                        let children: Vec<AnyElement> =
                            std::iter::once(overlay).chain(content(cx)).collect();
                        vec![cx.container(container, move |_cx| children)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}
