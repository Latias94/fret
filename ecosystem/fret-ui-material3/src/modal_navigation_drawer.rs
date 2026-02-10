//! Material 3 modal navigation drawer (overlay MVP).
//!
//! Outcome-oriented implementation:
//! - Uses `OverlayRequest::modal` to install a modal barrier (no click-through).
//! - Token-driven scrim color/opacity (`md.comp.navigation-drawer.scrim.*`).
//! - Slide-in drawer motion driven by the theme's cubic-bezier easing tokens.
//! - Focus is trapped within the drawer while open; focus is restored on close (via overlay infra).

use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::{DismissReason, DismissRequestCx, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, FractionalRenderTransformProps, InsetStyle, InteractivityGateProps,
    LayoutStyle, Length, PositionStyle, PressableA11y, PressableProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion;

#[derive(Clone)]
pub struct ModalNavigationDrawer {
    open: Model<bool>,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ModalNavigationDrawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalNavigationDrawer")
            .field("open", &"<model>")
            .field("open_duration_ms", &self.open_duration_ms)
            .field("close_duration_ms", &self.close_duration_ms)
            .field("easing_key", &self.easing_key)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl ModalNavigationDrawer {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            open_duration_ms: None,
            close_duration_ms: None,
            easing_key: Some(Arc::<str>::from("md.sys.motion.easing.emphasized")),
            on_dismiss_request: None,
            test_id: None,
        }
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

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        drawer: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let open_now = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let (open_ms, close_ms, bezier, scrim_color_base, scrim_opacity, drawer_w) = {
                let theme = Theme::global(&*cx.app);

                let open_ms = self
                    .open_duration_ms
                    .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                    .unwrap_or(300);
                let close_ms = self
                    .close_duration_ms
                    .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                    .unwrap_or(300);

                let easing_key = self
                    .easing_key
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));
                let bezier = theme.easing_by_key(easing_key.as_ref()).unwrap_or(
                    fret_ui::theme::CubicBezier {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 1.0,
                        y2: 1.0,
                    },
                );

                let tokens = MaterialTokenResolver::new(theme);
                let scrim_color_base = tokens.color_comp_or_sys(
                    "md.comp.navigation-drawer.scrim.color",
                    "md.sys.color.scrim",
                );
                let scrim_opacity = theme
                    .number_by_key("md.comp.navigation-drawer.scrim.opacity")
                    .unwrap_or(0.4);

                let drawer_w = theme
                    .metric_by_key("md.comp.navigation-drawer.container.width")
                    .unwrap_or(Px(360.0));

                (
                    open_ms,
                    close_ms,
                    bezier,
                    scrim_color_base,
                    scrim_opacity,
                    drawer_w,
                )
            };

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

            let content_el = content(cx);

            if presence.present {
                let scrim_alpha =
                    (scrim_color_base.a * scrim_opacity * transition.progress).clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_color_base, scrim_alpha);

                let dismiss_handler: OnDismissRequest =
                    self.on_dismiss_request.clone().unwrap_or_else(|| {
                        let open = self.open.clone();
                        Arc::new(move |host, action_cx, _cx: &mut DismissRequestCx| {
                            let _ = host.models_mut().update(&open, |v| *v = false);
                            host.request_redraw(action_cx.window);
                        })
                    });
                let dismiss_handler_for_request = dismiss_handler.clone();

                #[derive(Default)]
                struct DerivedTestId {
                    base: Option<Arc<str>>,
                    scrim: Option<Arc<str>>,
                }

                let scrim_test_id = cx.with_state(DerivedTestId::default, |st| {
                    if st.base.as_deref() != self.test_id.as_deref() {
                        st.base = self.test_id.clone();
                        st.scrim = st
                            .base
                            .as_ref()
                            .map(|id| Arc::from(format!("{}-scrim", id.as_ref())));
                    }
                    st.scrim.clone()
                });

                let root = cx.named("modal_navigation_drawer_root", |cx| {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout.overflow = fret_ui::element::Overflow::Visible;

                    cx.container(
                        ContainerProps {
                            layout,
                            ..Default::default()
                        },
                        move |cx| {
                            let scrim = cx.named("scrim", |cx| {
                                cx.pressable(
                                    PressableProps {
                                        enabled: open_now,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            test_id: scrim_test_id.clone(),
                                            ..Default::default()
                                        },
                                        layout: {
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
                                            l
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, _st| {
                                        if open_now {
                                            let on_activate: OnActivate = {
                                                let dismiss_handler = dismiss_handler.clone();
                                                Arc::new(move |host, action_cx, _reason| {
                                                    let mut dismiss_cx = DismissRequestCx::new(
                                                        DismissReason::OutsidePress {
                                                            pointer: None,
                                                        },
                                                    );
                                                    dismiss_handler(
                                                        host,
                                                        action_cx,
                                                        &mut dismiss_cx,
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

                            let drawer_panel = cx.named("drawer_panel", |cx| {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Absolute;
                                layout.size.width = Length::Px(drawer_w);
                                layout.size.height = Length::Fill;
                                layout.inset = InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: None,
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                };
                                layout.overflow = fret_ui::element::Overflow::Visible;

                                let translate_x_fraction = transition.progress - 1.0;

                                cx.fractional_render_transform_props(
                                    FractionalRenderTransformProps {
                                        layout,
                                        translate_x_fraction,
                                        translate_y_fraction: 0.0,
                                    },
                                    move |cx| {
                                        vec![cx.interactivity_gate_props(
                                            InteractivityGateProps {
                                                layout: LayoutStyle::default(),
                                                present: true,
                                                interactive: open_now,
                                            },
                                            move |cx| {
                                                vec![focus_scope_prim::focus_trap(cx, move |cx| {
                                                    vec![drawer(cx)]
                                                })]
                                            },
                                        )]
                                    },
                                )
                            });

                            vec![scrim, drawer_panel]
                        },
                    )
                });

                let overlay_id = cx.root_id();
                let mut request = overlay_controller::OverlayRequest::modal(
                    overlay_id,
                    None,
                    self.open.clone(),
                    presence,
                    vec![root],
                );
                request.root_name = Some(format!(
                    "material3.modal_navigation_drawer.{}",
                    overlay_id.0
                ));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = Some(dismiss_handler_for_request);
                OverlayController::request(cx, request);
            }

            content_el
        })
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}
