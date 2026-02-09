//! Material 3 checkbox (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.checkbox.*`.
//! - State layer (hover/pressed/focus) + unbounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, SvgFit};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetStateProperty, WidgetStates, resolve_override_slot_opt_with,
    resolve_override_slot_with,
};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::tokens::checkbox as checkbox_tokens;

#[derive(Debug, Clone, Default)]
pub struct CheckboxStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub icon_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl CheckboxStyle {
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

    pub fn icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.icon_color = Some(color);
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
        if other.icon_color.is_some() {
            self.icon_color = other.icon_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Checkbox {
    checked: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
    style: CheckboxStyle,
}

impl std::fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
            .field("style", &self.style)
            .finish()
    }
}

impl Checkbox {
    pub fn new(checked: Model<bool>) -> Self {
        Self {
            checked,
            disabled: false,
            a11y_label: None,
            test_id: None,
            on_activate: None,
            style: CheckboxStyle::default(),
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

    /// Called after the checkbox toggles its `Model<bool>`.
    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn style(mut self, style: CheckboxStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let size = {
                let theme = Theme::global(&*cx.app);
                checkbox_tokens::size_tokens(theme)
            };

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;

                cx.key_add_on_key_down_for(pressable_id, consume_enter_key_handler());

                let checked_model_for_toggle = self.checked.clone();
                let enabled_for_toggle = enabled;
                let user_activate = self.on_activate.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    if enabled_for_toggle {
                        let _ = host.update_model(&checked_model_for_toggle, |v| *v = !*v);
                        host.request_redraw(action_cx.window);
                    }
                    if let Some(h) = user_activate.as_ref() {
                        h(host, action_cx, reason);
                    }
                }));

                let checked = cx
                    .get_model_copied(&self.checked, Invalidation::Layout)
                    .unwrap_or(false);

                let (corner_radii, layout, focus_ring) = {
                    let theme = Theme::global(&*cx.app);
                    let corner_radii = theme
                        .corners_by_key("md.sys.shape.corner.full")
                        .unwrap_or_else(|| Corners::all(Px(9999.0)));

                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.overflow = Overflow::Visible;
                    enforce_minimum_interactive_size(&mut layout, theme);

                    let focus_ring =
                        material_focus_ring_for_component(theme, "md.comp.checkbox", corner_radii);

                    (corner_radii, layout, focus_ring)
                };
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Checkbox),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(checked),
                        ..Default::default()
                    },
                    layout,
                    focus_ring: Some(focus_ring),
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

                        let checked = cx
                            .get_model_copied(&self.checked, Invalidation::Paint)
                            .unwrap_or(false);

                        let mut states = WidgetStates::from_pressable(cx, st, enabled);
                        if checked {
                            states |= WidgetStates::SELECTED;
                        }

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);

                        let (
                            chrome,
                            state_layer_target,
                            state_layer_color,
                            ripple_base_opacity,
                            config,
                        ) = {
                            let theme = Theme::global(&*cx.app);

                            let mut chrome =
                                checkbox_tokens::chrome(theme, checked, enabled, interaction);
                            let token_container_bg = chrome.container_bg;
                            chrome.container_bg = resolve_override_slot_opt_with(
                                self.style.container_background.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || token_container_bg,
                            );
                            let token_outline_color = chrome.outline_color;
                            chrome.outline_color = resolve_override_slot_opt_with(
                                self.style.outline_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || token_outline_color,
                            );
                            let token_icon_color = chrome.icon_color;
                            chrome.icon_color = resolve_override_slot_with(
                                self.style.icon_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || token_icon_color,
                            );

                            let state_layer_target = checkbox_tokens::state_layer_target_opacity(
                                theme,
                                checked,
                                enabled,
                                interaction,
                            );
                            let state_layer_color =
                                checkbox_tokens::state_layer_color(theme, checked, interaction);
                            let state_layer_color = resolve_override_slot_with(
                                self.style.state_layer_color.as_ref(),
                                states,
                                |color| color.resolve(theme),
                                || state_layer_color,
                            );

                            let ripple_base_opacity =
                                checkbox_tokens::pressed_state_layer_opacity(theme, checked);
                            let config = material_pressable_indication_config(
                                theme,
                                Some(Px(size.state_layer.0 * 0.5)),
                            );

                            (
                                chrome,
                                state_layer_target,
                                state_layer_color,
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

                        let content = checkbox_content(cx, size, chrome);
                        let chrome = material_checkbox_chrome(cx, size, vec![overlay, content]);

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn interaction_state(
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> checkbox_tokens::CheckboxInteraction {
    match pressable_interaction(pressed, hovered, focused) {
        Some(PressableInteraction::Pressed) => checkbox_tokens::CheckboxInteraction::Pressed,
        Some(PressableInteraction::Focused) => checkbox_tokens::CheckboxInteraction::Focused,
        Some(PressableInteraction::Hovered) => checkbox_tokens::CheckboxInteraction::Hovered,
        None => checkbox_tokens::CheckboxInteraction::None,
    }
}

type CheckboxChrome = checkbox_tokens::CheckboxChrome;
type CheckboxSizeTokens = checkbox_tokens::CheckboxSizeTokens;

fn material_checkbox_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: CheckboxSizeTokens,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.corner_radii = Corners::all(Px(size.state_layer.0 * 0.5));
    props.layout.size.width = Length::Px(size.state_layer);
    props.layout.size.height = Length::Px(size.state_layer);
    cx.container(props, move |_cx| children)
}

fn checkbox_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: CheckboxSizeTokens,
    chrome: CheckboxChrome,
) -> AnyElement {
    let box_el = checkbox_box(cx, size, chrome);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = Length::Px(size.state_layer);
    layout.size.height = Length::Px(size.state_layer);

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![box_el],
    )
}

fn checkbox_box<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: CheckboxSizeTokens,
    chrome: CheckboxChrome,
) -> AnyElement {
    let corner_radii = Corners::all(size.container_corner);

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(size.container);
    props.layout.size.height = Length::Px(size.container);
    props.corner_radii = corner_radii;
    props.background = chrome.container_bg;
    props.border = Edges::all(chrome.outline_width);
    props.border_color = chrome.outline_color;
    props.snap_to_device_pixels = true;

    cx.container(props, move |cx| {
        if chrome.container_bg.is_some() {
            let icon = material_icon(
                cx,
                &fret_icons::ids::ui::CHECK,
                size.icon,
                chrome.icon_color,
            );
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            vec![cx.flex(
                FlexProps {
                    layout,
                    direction: Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| vec![icon],
            )]
        } else {
            Vec::new()
        }
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

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}
