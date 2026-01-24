//! Material 3 checkbox (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.checkbox.*`.
//! - State layer (hover/pressed/focus) + unbounded ripple using `fret_ui::paint`.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, KeyCode, Px, SemanticsRole, SvgFit};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};

use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    IndicationConfig, RippleClip, advance_indication_for_pressable, material_ink_layer,
};

#[derive(Clone)]
pub struct Checkbox {
    checked: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
}

impl std::fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("on_activate", &self.on_activate.is_some())
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let size = checkbox_size_tokens(&theme);

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

                let corner_radii = Corners::all(Px(9999.0));
                let pressable_props = PressableProps {
                    enabled,
                    focusable: enabled,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Checkbox),
                        label: self.a11y_label.clone(),
                        test_id: self.test_id.clone(),
                        checked: Some(
                            cx.get_model_copied(&self.checked, Invalidation::Layout)
                                .unwrap_or(false),
                        ),
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l
                    },
                    focus_ring: Some(material_focus_ring_for_component(
                        &theme,
                        "md.comp.checkbox",
                        corner_radii,
                    )),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
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

                        let interaction = interaction_state(is_pressed, is_hovered, is_focused);
                        let chrome = checkbox_chrome(&theme, checked, enabled, interaction);

                        let state_layer_target = checkbox_state_layer_target_opacity(
                            &theme, enabled, is_pressed, is_hovered, is_focused,
                        );
                        let state_layer_color =
                            checkbox_state_layer_color(&theme, checked, interaction);

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

                        let ripple_base_opacity = checkbox_ripple_base_opacity(&theme, checked);
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
                            Corners::all(Px(9999.0)),
                            RippleClip::Unbounded,
                            state_layer_color,
                            indication.state_layer_opacity,
                            indication.ripple_frame,
                            indication.want_frames,
                        );

                        let content = checkbox_content(cx, size, chrome);
                        let chrome = material_checkbox_chrome(cx, size, vec![overlay, content]);

                        vec![chrome]
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
struct CheckboxSizeTokens {
    container: Px,
    icon: Px,
    state_layer: Px,
    container_corner: Px,
}

fn checkbox_size_tokens(theme: &Theme) -> CheckboxSizeTokens {
    let container = theme
        .metric_by_key("md.comp.checkbox.container.size")
        .unwrap_or(Px(18.0));
    let icon = theme
        .metric_by_key("md.comp.checkbox.icon.size")
        .unwrap_or(container);
    let state_layer = theme
        .metric_by_key("md.comp.checkbox.state-layer.size")
        .unwrap_or(Px(40.0));
    let container_corner = theme
        .metric_by_key("md.comp.checkbox.container.shape")
        .unwrap_or(Px(2.0));

    CheckboxSizeTokens {
        container,
        icon,
        state_layer,
        container_corner,
    }
}

fn checkbox_state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    pressed: bool,
    hovered: bool,
    focused: bool,
) -> f32 {
    if !enabled {
        return 0.0;
    }
    if pressed {
        return theme
            .number_by_key("md.sys.state.pressed.state-layer-opacity")
            .unwrap_or(0.1);
    }
    if focused {
        return theme
            .number_by_key("md.sys.state.focus.state-layer-opacity")
            .unwrap_or(0.1);
    }
    if hovered {
        return theme
            .number_by_key("md.sys.state.hover.state-layer-opacity")
            .unwrap_or(0.08);
    }
    0.0
}

fn checkbox_state_layer_color(theme: &Theme, checked: bool, interaction: Interaction) -> Color {
    let (group, suffix) = if checked {
        (
            "md.comp.checkbox.selected",
            match interaction {
                Interaction::Pressed => "pressed.state-layer.color",
                Interaction::Focused => "focus.state-layer.color",
                Interaction::Hovered => "hover.state-layer.color",
                Interaction::None => "hover.state-layer.color",
            },
        )
    } else {
        (
            "md.comp.checkbox.unselected",
            match interaction {
                Interaction::Pressed => "pressed.state-layer.color",
                Interaction::Focused => "focus.state-layer.color",
                Interaction::Hovered => "hover.state-layer.color",
                Interaction::None => "hover.state-layer.color",
            },
        )
    };

    theme
        .color_by_key(&format!("{group}.{suffix}"))
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

fn checkbox_ripple_base_opacity(theme: &Theme, checked: bool) -> f32 {
    let key = if checked {
        "md.comp.checkbox.selected.pressed.state-layer.opacity"
    } else {
        "md.comp.checkbox.unselected.pressed.state-layer.opacity"
    };

    theme.number_by_key(key).unwrap_or_else(|| {
        theme
            .number_by_key("md.sys.state.pressed.state-layer-opacity")
            .unwrap_or(0.1)
    })
}

#[derive(Debug, Clone, Copy)]
struct CheckboxChrome {
    container_bg: Option<Color>,
    outline_width: Px,
    outline_color: Option<Color>,
    icon_color: Color,
}

fn checkbox_chrome(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: Interaction,
) -> CheckboxChrome {
    if checked {
        let mut container = theme
            .color_by_key("md.comp.checkbox.selected.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.primary"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.primary"));
        let mut icon_color = theme
            .color_by_key("md.comp.checkbox.selected.icon.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-primary"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-primary"));

        if !enabled {
            let opacity = theme
                .number_by_key("md.comp.checkbox.selected.disabled.container.opacity")
                .or_else(|| theme.number_by_key("md.sys.state.disabled.state-layer-opacity"))
                .unwrap_or(0.38);
            container = alpha_mul(
                theme
                    .color_by_key("md.comp.checkbox.selected.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or(container),
                opacity,
            );
            icon_color = theme
                .color_by_key("md.comp.checkbox.selected.disabled.icon.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface"))
                .unwrap_or(icon_color);
        }

        CheckboxChrome {
            container_bg: Some(container),
            outline_width: theme
                .metric_by_key("md.comp.checkbox.selected.outline.width")
                .unwrap_or(Px(0.0)),
            outline_color: None,
            icon_color,
        }
    } else {
        let outline_width = if enabled {
            theme
                .metric_by_key("md.comp.checkbox.unselected.outline.width")
                .unwrap_or(Px(2.0))
        } else {
            theme
                .metric_by_key("md.comp.checkbox.unselected.disabled.outline.width")
                .unwrap_or(Px(2.0))
        };

        let base_outline = match interaction {
            Interaction::Pressed => theme
                .color_by_key("md.comp.checkbox.unselected.pressed.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface")),
            Interaction::Focused => theme
                .color_by_key("md.comp.checkbox.unselected.focus.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface")),
            Interaction::Hovered => theme
                .color_by_key("md.comp.checkbox.unselected.hover.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface")),
            Interaction::None => theme
                .color_by_key("md.comp.checkbox.unselected.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant")),
        }
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"));

        let outline_color = if enabled {
            base_outline
        } else {
            let opacity = theme
                .number_by_key("md.comp.checkbox.unselected.disabled.container.opacity")
                .or_else(|| theme.number_by_key("md.sys.state.disabled.state-layer-opacity"))
                .unwrap_or(0.38);
            base_outline.map(|c| alpha_mul(c, opacity))
        };

        CheckboxChrome {
            container_bg: None,
            outline_width,
            outline_color,
            icon_color: theme
                .color_by_key("md.comp.checkbox.selected.icon.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-primary"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.on-primary")),
        }
    }
}

fn material_checkbox_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: CheckboxSizeTokens,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.corner_radii = Corners::all(Px(9999.0));
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

fn consume_enter_key_handler() -> fret_ui::action::OnKeyDown {
    Arc::new(|_host, _cx, down| matches!(down.key, KeyCode::Enter | KeyCode::NumpadEnter))
}
