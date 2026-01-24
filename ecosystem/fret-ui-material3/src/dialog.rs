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
use fret_ui::action::{DismissReason, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PointerRegionProps, PositionStyle, PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::indication::{
    IndicationConfig, advance_indication_for_pressable, material_ink_layer,
};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion;

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
        theme: &Theme,
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

                    let tokens = MaterialTokenResolver::new(theme);

                    let label_color = if is_pressed {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.pressed.label-text.color",
                            "md.sys.color.primary",
                        )
                    } else if is_hovered {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.hover.label-text.color",
                            "md.sys.color.primary",
                        )
                    } else if is_focused {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.focus.label-text.color",
                            "md.sys.color.primary",
                        )
                    } else {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.label-text.color",
                            "md.sys.color.primary",
                        )
                    };

                    let state_layer_color = if is_pressed {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.pressed.state-layer.color",
                            "md.sys.color.primary",
                        )
                    } else if is_hovered {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.hover.state-layer.color",
                            "md.sys.color.primary",
                        )
                    } else {
                        tokens.color_comp_or_sys(
                            "md.comp.dialog.action.focus.state-layer.color",
                            "md.sys.color.primary",
                        )
                    };

                    let state_layer_target = if is_pressed {
                        theme
                            .number_by_key("md.comp.dialog.action.pressed.state-layer.opacity")
                            .unwrap_or(0.1)
                    } else if is_hovered {
                        theme
                            .number_by_key("md.comp.dialog.action.hover.state-layer.opacity")
                            .unwrap_or(0.08)
                    } else if is_focused {
                        theme
                            .number_by_key("md.comp.dialog.action.focus.state-layer.opacity")
                            .unwrap_or(0.1)
                    } else {
                        0.0
                    };

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
                        .number_by_key("md.comp.dialog.action.pressed.state-layer.opacity")
                        .unwrap_or(0.1);
                    let indication = advance_indication_for_pressable(
                        cx,
                        pressable_id,
                        cx.frame_id.0,
                        bounds,
                        last_down,
                        is_pressed,
                        state_layer_target,
                        ripple_base_opacity,
                        IndicationConfig {
                            state_duration_ms,
                            ripple_expand_ms,
                            ripple_fade_ms,
                            easing,
                        },
                    );

                    let ink = material_ink_layer(
                        cx,
                        config.corner_radii,
                        state_layer_color,
                        indication.state_layer_opacity,
                        indication.ripple_frame,
                        indication.want_frames,
                    );

                    let label_style = theme
                        .text_style_by_key("md.sys.typescale.label-large")
                        .unwrap_or_else(|| {
                            let mut style = TextStyle::default();
                            style.size = Px(14.0);
                            style.weight = fret_core::FontWeight::MEDIUM;
                            style
                        });

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
        }
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        underlay: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let open_now = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let open_ms = self
                .open_duration_ms
                .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                .unwrap_or(300);
            let close_ms = self
                .close_duration_ms
                .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                .unwrap_or(300);
            let open_ticks = motion::ms_to_frames(open_ms);
            let close_ticks = motion::ms_to_frames(close_ms);
            let easing_key = self
                .easing_key
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));
            let bezier =
                theme
                    .easing_by_key(easing_key.as_ref())
                    .unwrap_or(fret_ui::theme::CubicBezier {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 1.0,
                        y2: 1.0,
                    });

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
                let tokens = MaterialTokenResolver::new(&theme);

                let dismiss_handler: OnDismissRequest =
                    self.on_dismiss_request.clone().unwrap_or_else(|| {
                        let open = self.open.clone();
                        Arc::new(move |host, action_cx, _reason: DismissReason| {
                            let _ = host.models_mut().update(&open, |v| *v = false);
                            host.request_redraw(action_cx.window);
                        })
                    });
                let dismiss_handler_for_request = dismiss_handler.clone();

                let scrim_color = tokens.color_sys("md.sys.color.scrim");
                let scrim_alpha =
                    (scrim_color.a * self.scrim_opacity * transition.progress).clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_color, scrim_alpha);

                let container_bg = tokens.color_comp_or_sys(
                    "md.comp.dialog.container.color",
                    "md.sys.color.surface-container-high",
                );
                let container_shape = theme
                    .corners_by_key("md.comp.dialog.container.shape")
                    .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
                    .unwrap_or_else(|| Corners::all(Px(28.0)));
                let elevation = theme
                    .metric_by_key("md.comp.dialog.container.elevation")
                    .unwrap_or(Px(0.0));
                let shadow_color = theme
                    .color_by_key("md.comp.dialog.container.shadow-color")
                    .unwrap_or_else(|| tokens.color_sys("md.sys.color.shadow"));
                let shadow = shadow_for_elevation_with_color(
                    &theme,
                    elevation,
                    Some(shadow_color),
                    container_shape,
                );

                let headline_color = tokens
                    .color_comp_or_sys("md.comp.dialog.headline.color", "md.sys.color.on-surface");
                let supporting_color = tokens.color_comp_or_sys(
                    "md.comp.dialog.supporting-text.color",
                    "md.sys.color.on-surface-variant",
                );

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
                                            test_id: self
                                                .test_id
                                                .clone()
                                                .map(|id| Arc::from(format!("{id}-scrim"))),
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
                                                    dismiss_handler(
                                                        host,
                                                        action_cx,
                                                        DismissReason::OutsidePress,
                                                    );
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
                                let translate_y = Px((1.0 - transition.progress) * 20.0);

                                let mut center_layout = LayoutStyle::default();
                                center_layout.position = PositionStyle::Absolute;
                                center_layout.size.width = Length::Fill;
                                center_layout.size.height = Length::Fill;
                                center_layout.inset = InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                };

                                let mut center = FlexProps::default();
                                center.layout = center_layout;
                                center.direction = Axis::Vertical;
                                center.justify = MainAlign::Center;
                                center.align = CrossAlign::Center;
                                center.padding = Edges::all(Px(24.0));

                                cx.render_transform(
                                    fret_core::Transform2D::translation(fret_core::Point::new(
                                        Px(0.0),
                                        translate_y,
                                    )),
                                    move |cx| {
                                        vec![cx.opacity(transition.progress, move |cx| {
                                            vec![cx.flex(center, move |cx| {
                                                let mut panel_layout = LayoutStyle::default();
                                                panel_layout.size.width = Length::Fill;
                                                panel_layout.size.max_width = Some(Px(560.0));
                                                panel_layout.size.min_width = Some(Px(280.0));
                                                panel_layout.overflow = Overflow::Clip;

                                                let mut body = Vec::new();
                                                if let Some(headline) = self.headline.clone() {
                                                    let style = theme
                                                        .text_style_by_key(
                                                            "md.sys.typescale.headline-small",
                                                        )
                                                        .unwrap_or_else(|| {
                                                            let mut style = TextStyle::default();
                                                            style.size = Px(24.0);
                                                            style
                                                        });
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
                                                    let style = theme
                                                        .text_style_by_key(
                                                            "md.sys.typescale.body-medium",
                                                        )
                                                        .unwrap_or_else(|| {
                                                            let mut style = TextStyle::default();
                                                            style.size = Px(14.0);
                                                            style
                                                        });
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

                                                    let action_cfg = DialogActionConfig {
                                                        height: Px(40.0),
                                                        padding: Edges {
                                                            left: Px(12.0),
                                                            right: Px(12.0),
                                                            top: Px(0.0),
                                                            bottom: Px(0.0),
                                                        },
                                                        corner_radii: Corners::all(Px(9999.0)),
                                                    };

                                                    let actions = self
                                                        .actions
                                                        .clone()
                                                        .into_iter()
                                                        .map(|a| {
                                                            a.into_element(cx, &theme, action_cfg)
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
                                                            padding: Edges::all(Px(24.0)),
                                                            ..Default::default()
                                                        },
                                                        move |_cx| body,
                                                    )]
                                                })]
                                            })]
                                        })]
                                    },
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
