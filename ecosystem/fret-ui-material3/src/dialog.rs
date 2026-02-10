//! Material 3 dialog (modal overlay MVP).
//!
//! Outcome-oriented implementation:
//! - Uses `OverlayRequest::modal` to install a modal barrier (no click-through).
//! - Token-driven container styling via `md.comp.dialog.*` (with sys fallbacks).
//! - Scrim uses `md.sys.color.scrim` with a Material-aligned default opacity.
//! - Focus is trapped within the dialog while open; focus is restored on close (via overlay infra).

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::action::{DismissReason, DismissRequestCx, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PointerRegionProps, PositionStyle, PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{
    ColorRef, OverlayController, OverlayPresence, OverrideSlot, WidgetStateProperty, WidgetStates,
    merge_override_slot, resolve_override_slot_with,
};

use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::surface::material_surface_style;
use crate::motion;
use crate::tokens::dialog as dialog_tokens;

#[derive(Debug, Clone, Default)]
pub struct DialogStyle {
    pub scrim_color: OverrideSlot<ColorRef>,
    pub container_background: OverrideSlot<ColorRef>,
    pub container_corner_radii: OverrideSlot<Corners>,
    pub container_elevation: OverrideSlot<Px>,
    pub headline_color: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
}

impl DialogStyle {
    pub fn scrim_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.scrim_color = Some(color);
        self
    }

    pub fn container_background(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.container_background = Some(color);
        self
    }

    pub fn container_corner_radii(mut self, corners: WidgetStateProperty<Option<Corners>>) -> Self {
        self.container_corner_radii = Some(corners);
        self
    }

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
        self
    }

    pub fn headline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.headline_color = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        Self {
            scrim_color: merge_override_slot(self.scrim_color, other.scrim_color),
            container_background: merge_override_slot(
                self.container_background,
                other.container_background,
            ),
            container_corner_radii: merge_override_slot(
                self.container_corner_radii,
                other.container_corner_radii,
            ),
            container_elevation: merge_override_slot(
                self.container_elevation,
                other.container_elevation,
            ),
            headline_color: merge_override_slot(self.headline_color, other.headline_color),
            supporting_text_color: merge_override_slot(
                self.supporting_text_color,
                other.supporting_text_color,
            ),
        }
    }
}

#[derive(Clone)]
pub struct DialogAction {
    label: Arc<str>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DialogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogAction")
            .field("label", &self.label)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl DialogAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            on_activate: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        config: DialogActionConfig,
    ) -> AnyElement {
        let DialogAction {
            label,
            on_activate,
            disabled,
            a11y_label,
            test_id,
        } = self;

        cx.pressable_with_id_props(move |cx, st, pressable_id| {
            let enabled = !disabled;

            if enabled {
                if let Some(on_activate) = on_activate.clone() {
                    cx.pressable_on_activate(on_activate);
                }
            }

            let mut props = PressableProps {
                enabled,
                focusable: enabled,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: a11y_label.clone().or_else(|| Some(label.clone())),
                    test_id: test_id.clone(),
                    ..Default::default()
                },
                layout: {
                    let mut l = LayoutStyle::default();
                    l.size.height = Length::Px(config.height);
                    l.overflow = Overflow::Visible;
                    l
                },
                focus_ring: None,
                focus_ring_bounds: None,
            };

            props.layout.size.width = Length::Auto;

            let children = vec![cx.pointer_region(
                PointerRegionProps {
                    enabled,
                    ..Default::default()
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                    let focus_visible =
                        fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
                    let is_pressed = enabled && st.pressed;
                    let is_hovered = enabled && st.hovered;
                    let is_focused = enabled && st.focused && focus_visible;

                    let interaction = if is_pressed {
                        dialog_tokens::DialogActionInteraction::Pressed
                    } else if is_hovered {
                        dialog_tokens::DialogActionInteraction::Hovered
                    } else if is_focused {
                        dialog_tokens::DialogActionInteraction::Focused
                    } else {
                        dialog_tokens::DialogActionInteraction::Default
                    };

                    let (
                        label_color,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        indication_config,
                        label_style,
                    ) = {
                        let theme = Theme::global(&*cx.app);
                        let label_color = dialog_tokens::action_label_color(theme, interaction);
                        let state_layer_color =
                            dialog_tokens::action_state_layer_color(theme, interaction);
                        let state_layer_target =
                            dialog_tokens::action_state_layer_target_opacity(theme, interaction);

                        let ripple_base_opacity =
                            dialog_tokens::action_pressed_state_layer_opacity(theme);
                        let indication_config = material_pressable_indication_config(theme, None);

                        let label_style = theme
                            .text_style_by_key("md.sys.typescale.label-large")
                            .unwrap_or_else(|| {
                                let mut style = TextStyle::default();
                                style.size = Px(14.0);
                                style.weight = fret_core::FontWeight::MEDIUM;
                                style
                            });

                        (
                            label_color,
                            state_layer_color,
                            state_layer_target,
                            ripple_base_opacity,
                            indication_config,
                            label_style,
                        )
                    };
                    let ink = material_ink_layer_for_pressable(
                        cx,
                        pressable_id,
                        cx.frame_id.0,
                        config.corner_radii,
                        RippleClip::Bounded,
                        state_layer_color,
                        is_pressed,
                        state_layer_target,
                        ripple_base_opacity,
                        indication_config,
                        false,
                    );

                    let text = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: label.clone(),
                        style: Some(label_style),
                        color: Some(label_color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    });

                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut l = LayoutStyle::default();
                                l.size.width = Length::Auto;
                                l.size.height = Length::Fill;
                                l
                            },
                            padding: config.padding,
                            ..Default::default()
                        },
                        move |_cx| vec![text, ink],
                    )]
                },
            )];

            (props, children)
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct DialogActionConfig {
    height: Px,
    padding: Edges,
    corner_radii: Corners,
}

#[derive(Clone)]
pub struct Dialog {
    open: Model<bool>,
    headline: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    actions: Vec<DialogAction>,
    scrim_opacity: f32,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    test_id: Option<Arc<str>>,
    style: DialogStyle,
}

impl std::fmt::Debug for Dialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialog")
            .field("open", &"<model>")
            .field("headline", &self.headline)
            .field("supporting_text", &self.supporting_text)
            .field("actions_len", &self.actions.len())
            .field("scrim_opacity", &self.scrim_opacity)
            .field("open_duration_ms", &self.open_duration_ms)
            .field("close_duration_ms", &self.close_duration_ms)
            .field("easing_key", &self.easing_key)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Dialog {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            headline: None,
            supporting_text: None,
            actions: Vec::new(),
            // Material guidance defaults around ~0.32 for modal scrims; Material Web exposes
            // navigation-drawer scrim opacity as a token, but dialog does not (v30 sassvars).
            scrim_opacity: 0.32,
            open_duration_ms: None,
            close_duration_ms: None,
            easing_key: Some(Arc::<str>::from("md.sys.motion.easing.emphasized")),
            on_dismiss_request: None,
            test_id: None,
            style: DialogStyle::default(),
        }
    }

    pub fn style(mut self, style: DialogStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn headline(mut self, text: impl Into<Arc<str>>) -> Self {
        self.headline = Some(text.into());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn actions(mut self, actions: Vec<DialogAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn open_duration_ms(mut self, ms: Option<u32>) -> Self {
        self.open_duration_ms = ms;
        self
    }

    pub fn close_duration_ms(mut self, ms: Option<u32>) -> Self {
        self.close_duration_ms = ms;
        self
    }

    pub fn easing_key(mut self, key: Option<impl Into<Arc<str>>>) -> Self {
        self.easing_key = key.map(Into::into);
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        underlay: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        cx.scope(|cx| {
            let open_now = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let easing_key = self
                .easing_key
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));

            let (open_ms_default, close_ms_default, bezier) = {
                let theme = Theme::global(&*cx.app);
                let open_ms = dialog_tokens::default_open_duration_ms(theme);
                let close_ms = dialog_tokens::default_close_duration_ms(theme);
                let bezier = dialog_tokens::easing(theme, Some(easing_key.as_ref()));
                (open_ms, close_ms, bezier)
            };

            let open_ms = self
                .open_duration_ms
                .unwrap_or(open_ms_default);
            let close_ms = self
                .close_duration_ms
                .unwrap_or(close_ms_default);
            let open_ticks = motion::ms_to_frames(open_ms);
            let close_ticks = motion::ms_to_frames(close_ms);

            let transition = OverlayController::transition_with_durations_and_cubic_bezier(
                cx,
                open_now,
                open_ticks,
                close_ticks,
                bezier,
            );
            let presence = OverlayPresence {
                present: transition.present,
                interactive: open_now,
            };

            let underlay_el = underlay(cx);

            if presence.present {
                let dismiss_handler: OnDismissRequest =
                    self.on_dismiss_request.clone().unwrap_or_else(|| {
                        let open = self.open.clone();
                        Arc::new(move |host, action_cx, _cx: &mut DismissRequestCx| {
                            let _ = host.models_mut().update(&open, |v| *v = false);
                            host.request_redraw(action_cx.window);
                        })
                    });
                let dismiss_handler_for_request = dismiss_handler.clone();

                let (
                    scrim_color,
                    container_bg,
                    container_shape,
                    shadow,
                    headline_color,
                    supporting_color,
                    headline_style,
                    supporting_style,
                    action_cfg,
                    panel_padding,
                ) = {
                    let theme = Theme::global(&*cx.app);

                    let scrim_color = resolve_override_slot_with(
                        self.style.scrim_color.as_ref(),
                        WidgetStates::empty(),
                        |color| color.resolve(theme),
                        || dialog_tokens::scrim_color(theme),
                    );
                    let scrim_alpha = (scrim_color.a
                        * self.scrim_opacity
                        * transition.progress)
                        .clamp(0.0, 1.0);
                    let scrim_color = with_alpha(scrim_color, scrim_alpha);

                    let container_bg = resolve_override_slot_with(
                        self.style.container_background.as_ref(),
                        WidgetStates::empty(),
                        |color| color.resolve(theme),
                        || dialog_tokens::container_background(theme),
                    );
                    let container_shape = resolve_override_slot_with(
                        self.style.container_corner_radii.as_ref(),
                        WidgetStates::empty(),
                        |v| *v,
                        || dialog_tokens::container_shape(theme),
                    );
                    let elevation = resolve_override_slot_with(
                        self.style.container_elevation.as_ref(),
                        WidgetStates::empty(),
                        |v| *v,
                        || dialog_tokens::container_elevation(theme),
                    );
                    let shadow_color = dialog_tokens::container_shadow_color(theme);
                    let surface = material_surface_style(
                        theme,
                        container_bg,
                        elevation,
                        Some(shadow_color),
                        container_shape,
                    );
                    let container_bg = surface.background;
                    let shadow = surface.shadow;

                    let headline_color = resolve_override_slot_with(
                        self.style.headline_color.as_ref(),
                        WidgetStates::empty(),
                        |color| color.resolve(theme),
                        || dialog_tokens::headline_color(theme),
                    );
                    let supporting_color = resolve_override_slot_with(
                        self.style.supporting_text_color.as_ref(),
                        WidgetStates::empty(),
                        |color| color.resolve(theme),
                        || dialog_tokens::supporting_text_color(theme),
                    );

                    let headline_style = theme
                        .text_style_by_key("md.sys.typescale.headline-small")
                        .unwrap_or_else(|| {
                            let mut style = TextStyle::default();
                            style.size = Px(24.0);
                            style
                        });
                    let supporting_style = theme
                        .text_style_by_key("md.sys.typescale.body-medium")
                        .unwrap_or_else(|| {
                            let mut style = TextStyle::default();
                            style.size = Px(14.0);
                            style
                        });

                    let action_cfg = DialogActionConfig {
                        height: dialog_tokens::action_height(theme),
                        padding: dialog_tokens::action_padding(theme),
                        corner_radii: dialog_tokens::action_corner_radii(theme),
                    };
                    let panel_padding = dialog_tokens::panel_padding(theme);

                    (
                        scrim_color,
                        container_bg,
                        container_shape,
                        shadow,
                        headline_color,
                        supporting_color,
                        headline_style,
                        supporting_style,
                        action_cfg,
                        panel_padding,
                    )
                };

                let overlay_root = cx.named("material3_dialog_root", |cx| {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout.overflow = Overflow::Visible;

                    cx.container(
                        ContainerProps {
                            layout,
                            ..Default::default()
                        },
                        move |cx| {
                            let scrim = cx.named("scrim", |cx| {
                                #[derive(Default)]
                                struct DerivedTestId {
                                    base: Option<Arc<str>>,
                                    scrim: Option<Arc<str>>,
                                }

                                let scrim_test_id = cx.with_state(DerivedTestId::default, |st| {
                                    if st.base.as_deref() != self.test_id.as_deref() {
                                        st.base = self.test_id.clone();
                                        st.scrim = st.base.as_ref().map(|id| {
                                            Arc::from(format!("{}-scrim", id.as_ref()))
                                        });
                                    }
                                    st.scrim.clone()
                                });

                                let mut l = LayoutStyle::default();
                                l.position = PositionStyle::Absolute;
                                l.size.width = Length::Fill;
                                l.size.height = Length::Fill;
                                l.inset = InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                };

                                cx.pressable(
                                    PressableProps {
                                        enabled: open_now,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            test_id: scrim_test_id,
                                            ..Default::default()
                                        },
                                        layout: l,
                                        ..Default::default()
                                    },
                                    move |cx, _st| {
                                        if open_now {
                                            let on_activate: OnActivate = {
                                                let dismiss_handler = dismiss_handler.clone();
                                                Arc::new(move |host, action_cx, _reason| {
                                                    let mut dismiss_cx = DismissRequestCx::new(
                                                        DismissReason::OutsidePress { pointer: None },
                                                    );
                                                    dismiss_handler(host, action_cx, &mut dismiss_cx);
                                                })
                                            };
                                            cx.pressable_on_activate(on_activate);
                                        }

                                        vec![cx.container(
                                            ContainerProps {
                                                background: Some(scrim_color),
                                                layout: {
                                                    let mut l = LayoutStyle::default();
                                                    l.size.width = Length::Fill;
                                                    l.size.height = Length::Fill;
                                                    l
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::<AnyElement>::new(),
                                        )]
                                    },
                                )
                            });

                            let panel = cx.named("panel", |cx| {
                                let opacity = transition.progress;
                                let translate_y = Px((1.0 - transition.progress) * 20.0);
                                let scale = 0.9 + 0.1 * transition.progress;

                                let origin = fret_core::Point::new(
                                    Px(cx.bounds.origin.x.0 + cx.bounds.size.width.0 * 0.5),
                                    Px(cx.bounds.origin.y.0 + cx.bounds.size.height.0 * 0.5),
                                );
                                let origin_inv =
                                    fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
                                let transform = fret_core::Transform2D::translation(
                                    fret_core::Point::new(Px(0.0), translate_y),
                                ) * fret_core::Transform2D::translation(origin)
                                    * fret_core::Transform2D::scale_uniform(scale)
                                    * fret_core::Transform2D::translation(origin_inv);

                                let mut center_layout = LayoutStyle::default();
                                center_layout.size.width = Length::Fill;
                                center_layout.size.height = Length::Fill;

                                let mut center = FlexProps::default();
                                center.layout = center_layout;
                                center.direction = Axis::Vertical;
                                center.justify = MainAlign::Center;
                                center.align = CrossAlign::Center;
                                center.padding = panel_padding;

                                let content = cx.flex(center, move |cx| {
                                                let mut panel_layout = LayoutStyle::default();
                                                panel_layout.size.width = Length::Fill;
                                                panel_layout.size.max_width = Some(Px(560.0));
                                                panel_layout.size.min_width = Some(Px(280.0));
                                                panel_layout.overflow = Overflow::Clip;

                                                let mut body = Vec::new();
                                                if let Some(headline) = self.headline.clone() {
                                                    let style = headline_style.clone();
                                                    body.push(cx.text_props(TextProps {
                                                        layout: LayoutStyle::default(),
                                                        text: headline,
                                                        style: Some(style),
                                                        color: Some(headline_color),
                                                        wrap: TextWrap::Word,
                                                        overflow: TextOverflow::Clip,
                                                    }));
                                                }
                                                if let Some(text) = self.supporting_text.clone() {
                                                    let style = supporting_style.clone();
                                                    body.push(cx.text_props(TextProps {
                                                        layout: LayoutStyle::default(),
                                                        text,
                                                        style: Some(style),
                                                        color: Some(supporting_color),
                                                        wrap: TextWrap::Word,
                                                        overflow: TextOverflow::Clip,
                                                    }));
                                                }

                                                body.extend(content(cx));

                                                if !self.actions.is_empty() {
                                                    let mut row = FlexProps::default();
                                                    row.direction = Axis::Horizontal;
                                                    row.justify = MainAlign::End;
                                                    row.align = CrossAlign::Center;
                                                    row.gap = Px(8.0);
                                                    row.layout.size.width = Length::Fill;

                                                    let actions = self
                                                        .actions
                                                        .clone()
                                                        .into_iter()
                                                        .map(|a| {
                                                            a.into_element(cx, action_cfg)
                                                        })
                                                        .collect::<Vec<_>>();

                                                    body.push(cx.flex(row, move |_cx| actions));
                                                }

                                                vec![focus_scope_prim::focus_trap(cx, move |cx| {
                                                    vec![cx.container(
                                                        ContainerProps {
                                                            layout: panel_layout,
                                                            background: Some(container_bg),
                                                            shadow,
                                                            corner_radii: container_shape,
                                                            padding: panel_padding,
                                                            ..Default::default()
                                                        },
                                                        move |_cx| body,
                                                    )]
                                                })]
                                            });

                                fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                                    cx,
                                    opacity,
                                    transform,
                                    presence.interactive,
                                    vec![content],
                                )
                            });

                            vec![scrim, panel]
                        },
                    )
                });

                let overlay_id = cx.root_id();
                let mut request = overlay_controller::OverlayRequest::modal(
                    overlay_id,
                    None,
                    self.open.clone(),
                    presence,
                    vec![overlay_root],
                );
                request.root_name = Some(format!("material3.dialog.{}", overlay_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = Some(dismiss_handler_for_request);
                OverlayController::request(cx, request);
            }

            underlay_el
        })
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}
