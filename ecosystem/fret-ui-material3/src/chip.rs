//! Material 3 assist chip (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven shape/colors via `md.comp.assist-chip.*` (v30 sassvars subset).
//! - State layer + bounded ripple using the shared Material foundation indication path.

use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px, SemanticsRole, SvgFit, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::pressable_interaction;
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::foundation::surface::material_surface_style;
use crate::tokens::chip as chip_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssistChipVariant {
    #[default]
    Flat,
    Elevated,
}

#[derive(Debug, Clone, Default)]
pub struct AssistChipStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub leading_icon_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl AssistChipStyle {
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

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn leading_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.leading_icon_color = Some(color);
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
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.leading_icon_color.is_some() {
            self.leading_icon_color = other.leading_icon_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct AssistChip {
    label: Arc<str>,
    leading_icon: Option<IconId>,
    variant: AssistChipVariant,
    on_activate: Option<OnActivate>,
    style: AssistChipStyle,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    roving_tab_stop: Option<bool>,
}

impl std::fmt::Debug for AssistChip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssistChip")
            .field("label", &self.label)
            .field("leading_icon", &self.leading_icon)
            .field("variant", &self.variant)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("roving_tab_stop", &self.roving_tab_stop)
            .finish()
    }
}

impl AssistChip {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            leading_icon: None,
            variant: AssistChipVariant::default(),
            on_activate: None,
            style: AssistChipStyle::default(),
            disabled: false,
            a11y_label: None,
            test_id: None,
            roving_tab_stop: None,
        }
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: AssistChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: AssistChipStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enable roving-focus-friendly tab stop behavior.
    ///
    /// When enabled, only the current tab stop (or the currently focused item) is included in the
    /// default focus traversal order.
    pub fn roving_tab_stop(mut self, tab_stop: bool) -> Self {
        self.roving_tab_stop = Some(tab_stop);
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

    pub(crate) fn disabled_for_roving(&self) -> bool {
        self.disabled
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;
                let focusable = match self.roving_tab_stop {
                    None => enabled,
                    Some(tab_stop) => enabled && (tab_stop || st.focused),
                };

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let (corner_radii, layout, focus_ring) = {
                    let theme = Theme::global(&*cx.app);
                    let corner_radii = chip_tokens::container_shape(theme);

                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.overflow = Overflow::Visible;
                    enforce_minimum_interactive_size(&mut layout, theme);

                    let focus_ring = material_focus_ring_for_component(
                        theme,
                        chip_tokens::COMPONENT_PREFIX,
                        corner_radii,
                    );

                    (corner_radii, layout, focus_ring)
                };

                let pressable_props = PressableProps {
                    enabled,
                    focusable,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: self.a11y_label.clone().or_else(|| Some(self.label.clone())),
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

                        let focus_visible =
                            fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                        let is_pressed = enabled && st.pressed;
                        let is_hovered = enabled && st.hovered;
                        let is_focused = enabled && st.focused && focus_visible;

                        let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                        let states = WidgetStates::from_pressable(cx, st, enabled);

                        let (
                            height,
                            label_style,
                            leading_icon_size,
                            label_color,
                            leading_icon_color,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            background,
                            shadow,
                            outline,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let height = chip_tokens::container_height(theme);
                            let label_style = theme
                                .text_style_by_key("md.sys.typescale.label-large")
                                .unwrap_or_else(|| fret_core::TextStyle::default());
                            let leading_icon_size = chip_tokens::leading_icon_size(theme);

                            let label_color = chip_tokens::label_color(theme, enabled, interaction);
                            let label_color = resolve_override_slot_with(
                                self.style.label_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || label_color,
                            );

                            let leading_icon_color =
                                chip_tokens::leading_icon_color(theme, enabled, interaction);
                            let leading_icon_color = resolve_override_slot_with(
                                self.style.leading_icon_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || leading_icon_color,
                            );

                            let state_layer_color =
                                chip_tokens::state_layer_color(theme, interaction);
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let state_layer_target =
                                chip_tokens::state_layer_opacity(theme, interaction);
                            let ripple_base_opacity =
                                chip_tokens::pressed_state_layer_opacity(theme);
                            let config = material_pressable_indication_config(theme, None);

                            let (background, shadow, outline) = match self.variant {
                                AssistChipVariant::Elevated => {
                                    let bg =
                                        chip_tokens::elevated_container_background(theme, enabled);
                                    let bg = resolve_override_slot_with(
                                        self.style.container_background.as_ref(),
                                        states,
                                        |color| color.resolve(theme),
                                        || bg,
                                    );
                                    let elevation = chip_tokens::elevated_container_elevation(
                                        theme,
                                        enabled,
                                        interaction,
                                    );
                                    let shadow_color =
                                        chip_tokens::elevated_container_shadow_color(theme);
                                    let surface = material_surface_style(
                                        theme,
                                        bg,
                                        elevation,
                                        Some(shadow_color),
                                        corner_radii,
                                    );
                                    (Some(surface.background), surface.shadow, None)
                                }
                                AssistChipVariant::Flat => {
                                    let outline =
                                        chip_tokens::flat_outline(theme, enabled, interaction);
                                    let outline = resolve_override_slot_opt_with(
                                        self.style.outline_color.as_ref(),
                                        states,
                                        |color| color.resolve(theme),
                                        || outline.map(|o| o.color),
                                    )
                                    .map(|color| chip_tokens::ChipOutline {
                                        color,
                                        width: outline.map(|o| o.width).unwrap_or(Px(0.0)),
                                    });
                                    (None, None, outline)
                                }
                            };

                            (
                                height,
                                label_style,
                                leading_icon_size,
                                label_color,
                                leading_icon_color,
                                state_layer_color,
                                state_layer_target,
                                ripple_base_opacity,
                                config,
                                background,
                                shadow,
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
                            false,
                        );

                        let content = assist_chip_content(
                            cx,
                            &self.label,
                            label_style,
                            label_color,
                            self.leading_icon,
                            leading_icon_size,
                            leading_icon_color,
                            height,
                        );

                        let mut chrome = ContainerProps::default();
                        chrome.layout.overflow = Overflow::Clip;
                        chrome.corner_radii = corner_radii;
                        chrome.background = background;
                        chrome.shadow = shadow;
                        chrome.layout.size.height = Length::Px(height);
                        if let Some(outline) = outline {
                            chrome.border = Edges::all(outline.width);
                            chrome.border_color = Some(outline.color);
                        }

                        let chrome = cx.container(chrome, move |_cx| vec![overlay, content]);

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn assist_chip_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &Arc<str>,
    label_style: fret_core::TextStyle,
    label_color: Color,
    leading_icon: Option<IconId>,
    leading_icon_size: Px,
    leading_icon_color: Color,
    height: Px,
) -> AnyElement {
    const LEADING_SPACE: Px = Px(16.0);
    const TRAILING_SPACE: Px = Px(16.0);
    const ICON_LABEL_SPACE: Px = Px(8.0);
    const WITH_LEADING_ICON_LEADING_SPACE: Px = Px(8.0);

    let mut text = TextProps::new(label.clone());
    text.style = Some(label_style);
    text.color = Some(label_color);
    text.wrap = TextWrap::None;
    text.overflow = TextOverflow::Ellipsis;

    let label_el = cx.text_props(text);

    let padding_left = if leading_icon.is_some() {
        WITH_LEADING_ICON_LEADING_SPACE
    } else {
        LEADING_SPACE
    };

    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props.gap = if leading_icon.is_some() {
        ICON_LABEL_SPACE
    } else {
        Px(0.0)
    };
    props.padding = Edges {
        left: padding_left,
        right: TRAILING_SPACE,
        top: Px(0.0),
        bottom: Px(0.0),
    };
    props.layout.size.height = Length::Px(height);

    cx.flex(props, move |cx| {
        let mut out = Vec::new();
        if let Some(icon) = leading_icon {
            out.push(material_icon(
                cx,
                &icon,
                leading_icon_size,
                leading_icon_color,
            ));
        }
        out.push(label_el);
        out
    })
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
