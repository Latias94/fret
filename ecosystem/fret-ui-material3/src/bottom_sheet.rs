//! Material 3 bottom sheet primitives (P2).
//!
//! Outcome-oriented implementation:
//! - Token-driven styling via `md.comp.sheet.bottom.*` (Material Web v30).
//! - Modal variant uses `OverlayRequest::modal` with a scrim and focus trap/restore.
//! - Standard variant is a docked container surface (non-overlay), suitable for scaffold-like layouts.

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{DismissReason, DismissRequestCx, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RingPlacement, RingStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::foundation::surface::material_surface_style;
use crate::motion;
use crate::tokens::sheet_bottom as sheet_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockedBottomSheetVariant {
    #[default]
    Standard,
    Modal,
}

#[derive(Clone)]
pub struct DockedBottomSheet {
    variant: DockedBottomSheetVariant,
    drag_handle: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DockedBottomSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockedBottomSheet")
            .field("variant", &self.variant)
            .field("drag_handle", &self.drag_handle)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl DockedBottomSheet {
    pub fn new() -> Self {
        Self {
            variant: DockedBottomSheetVariant::default(),
            drag_handle: true,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: DockedBottomSheetVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn drag_handle(mut self, enabled: bool) -> Self {
        self.drag_handle = enabled;
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
            let DockedBottomSheet {
                variant,
                drag_handle,
                test_id,
            } = self;

            let is_modal = variant == DockedBottomSheetVariant::Modal;
            let (surface, corner_radii, focus_ring) = {
                let theme = Theme::global(&*cx.app);
                let elevation = if is_modal {
                    sheet_tokens::docked_modal_elevation(theme)
                } else {
                    sheet_tokens::docked_standard_elevation(theme)
                };

                let background = sheet_tokens::docked_container_color(theme);
                let corner_radii = sheet_tokens::docked_container_shape(theme);
                let surface =
                    material_surface_style(theme, background, elevation, None, corner_radii);

                let focus_ring = RingStyle {
                    placement: RingPlacement::Outset,
                    width: sheet_tokens::focus_indicator_thickness(theme),
                    offset: sheet_tokens::focus_indicator_outline_offset(theme),
                    color: sheet_tokens::focus_indicator_color(theme),
                    offset_color: None,
                    corner_radii,
                };

                (surface, corner_radii, focus_ring)
            };

            let mut column = FlexProps::default();
            column.direction = Axis::Vertical;
            column.justify = MainAlign::Start;
            column.align = CrossAlign::Stretch;
            column.wrap = false;
            column.gap = Px(0.0);
            column.layout.size.width = Length::Fill;

            // Compose baseline: `SheetMaxWidth = 640.dp`.
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.max_width = Some(Px(640.0));
            layout.overflow = Overflow::Clip;

            let mut container = ContainerProps::default();
            container.layout = layout;
            container.background = Some(surface.background);
            container.shadow = surface.shadow;
            container.corner_radii = corner_radii;
            container.focus_within = true;
            container.focus_ring = Some(focus_ring);

            let test_id_for_children = test_id.clone();
            let content_el = cx.flex(column, move |cx| {
                let mut out: Vec<AnyElement> = Vec::new();
                if drag_handle {
                    out.push(drag_handle_element(cx, test_id_for_children.clone()));
                }
                out.extend(content(cx));
                out
            });

            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: test_id.clone(),
                    ..Default::default()
                },
                move |cx| vec![cx.container(container, move |_cx| vec![content_el])],
            )
        })
    }
}

#[derive(Clone)]
pub struct ModalBottomSheet {
    open: Model<bool>,
    scrim_opacity: f32,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    drag_handle: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ModalBottomSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalBottomSheet")
            .field("open", &"<model>")
            .field("scrim_opacity", &self.scrim_opacity)
            .field("open_duration_ms", &self.open_duration_ms)
            .field("close_duration_ms", &self.close_duration_ms)
            .field("easing_key", &self.easing_key)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("drag_handle", &self.drag_handle)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl ModalBottomSheet {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            // Align with Dialog defaults.
            scrim_opacity: 0.32,
            open_duration_ms: None,
            close_duration_ms: None,
            easing_key: Some(Arc::<str>::from("md.sys.motion.easing.emphasized")),
            on_dismiss_request: None,
            drag_handle: true,
            test_id: None,
        }
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

    pub fn drag_handle(mut self, enabled: bool) -> Self {
        self.drag_handle = enabled;
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
            let ModalBottomSheet {
                open,
                scrim_opacity,
                open_duration_ms,
                close_duration_ms,
                easing_key,
                on_dismiss_request,
                drag_handle,
                test_id,
            } = self;
            let open_now = cx
                .get_model_copied(&open, Invalidation::Layout)
                .unwrap_or(false);

            let (default_duration_ms, bezier, scrim_base) = {
                let theme = Theme::global(&*cx.app);
                let default_duration_ms = theme
                    .duration_ms_by_key("md.sys.motion.duration.medium2")
                    .unwrap_or(300);
                let easing_key =
                    easing_key.unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));
                let bezier =
                    theme
                        .easing_by_key(easing_key.as_ref())
                        .unwrap_or(fret_ui::theme::CubicBezier {
                            x1: 0.0,
                            y1: 0.0,
                            x2: 1.0,
                            y2: 1.0,
                        });
                let scrim_base = theme.color_required("md.sys.color.scrim");
                (default_duration_ms, bezier, scrim_base)
            };

            let open_ms = open_duration_ms.unwrap_or(default_duration_ms);
            let close_ms = close_duration_ms.unwrap_or(default_duration_ms);
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
                let scrim_alpha = (scrim_base.a * scrim_opacity * transition.progress)
                    .clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_base, scrim_alpha);

                let dismiss_handler: OnDismissRequest = on_dismiss_request.unwrap_or_else(|| {
                    let open = open.clone();
                    Arc::new(move |host, action_cx, _cx: &mut DismissRequestCx| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.request_redraw(action_cx.window);
                    })
                });
                let dismiss_handler_for_request = dismiss_handler.clone();

                let scrim_test_id = test_id.as_ref().map(|id| Arc::from(format!("{id}-scrim")));
                let sheet_test_id = test_id.as_ref().map(|id| Arc::from(format!("{id}-sheet")));

                let overlay_root = cx.named("modal_bottom_sheet_root", |cx| {
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
                                cx.pressable(
                                    PressableProps {
                                        enabled: open_now,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            test_id: scrim_test_id.clone(),
                                            ..Default::default()
                                        },
                                        layout: absolute_fill_layout(),
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
                                                layout: {
                                                    let mut l = LayoutStyle::default();
                                                    l.size.width = Length::Fill;
                                                    l.size.height = Length::Fill;
                                                    l
                                                },
                                                background: Some(scrim_color),
                                                ..Default::default()
                                            },
                                            |_cx| Vec::<AnyElement>::new(),
                                        )]
                                    },
                                )
                            });

                            let panel = cx.named("panel", |cx| {
                                let opacity = transition.progress;
                                let translate_y = Px((1.0 - transition.progress) * cx.bounds.size.height.0);
                                let transform = fret_core::Transform2D::translation(
                                    fret_core::Point::new(Px(0.0), translate_y),
                                );

                                let mut align = FlexProps::default();
                                align.direction = Axis::Vertical;
                                align.justify = MainAlign::End;
                                align.align = CrossAlign::Center;
                                align.wrap = false;
                                align.layout.size.width = Length::Fill;
                                align.layout.size.height = Length::Fill;

                                let docked = DockedBottomSheet::new()
                                    .variant(DockedBottomSheetVariant::Modal)
                                    .drag_handle(drag_handle)
                                    .test_id(sheet_test_id.clone().unwrap_or_else(|| {
                                        Arc::<str>::from("material3-modal-bottom-sheet")
                                    }));

                                let content_el = docked.into_element(cx, move |cx| content(cx));
                                let trapped = focus_scope_prim::focus_trap(cx, move |_cx| {
                                    vec![content_el]
                                });

                                let stacked = cx.flex(align, move |_cx| vec![trapped]);

                                fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                                    cx,
                                    opacity,
                                    transform,
                                    presence.interactive,
                                    vec![stacked],
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
                    open.clone(),
                    presence,
                    vec![overlay_root],
                );
                request.root_name = Some(format!("material3.modal_bottom_sheet.{}", overlay_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = Some(dismiss_handler_for_request);
                OverlayController::request(cx, request);
            }

            underlay_el
        })
    }
}

fn drag_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    sheet_test_id: Option<Arc<str>>,
) -> AnyElement {
    let (width, height, color) = {
        let theme = Theme::global(&*cx.app);
        let width = sheet_tokens::docked_drag_handle_width(theme);
        let height = sheet_tokens::docked_drag_handle_height(theme);
        let mut color = sheet_tokens::docked_drag_handle_color(theme);
        color.a = (color.a * sheet_tokens::docked_drag_handle_opacity(theme)).clamp(0.0, 1.0);
        (width, height, color)
    };

    // Compose baseline: `DragHandleVerticalPadding = 22.dp`.
    let padding_y = Px(22.0);

    let mut wrapper = ContainerProps::default();
    wrapper.layout.size.width = Length::Fill;
    wrapper.padding = Edges {
        left: Px(0.0),
        right: Px(0.0),
        top: padding_y,
        bottom: padding_y,
    };

    let mut row = FlexProps::default();
    row.direction = Axis::Horizontal;
    row.justify = MainAlign::Center;
    row.align = CrossAlign::Center;
    row.wrap = false;
    row.layout.size.width = Length::Fill;

    let handle = {
        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Px(width);
        props.layout.size.height = Length::Px(height);
        props.background = Some(color);
        props.corner_radii = Corners::all(Px(9999.0));
        cx.container(props, |_cx| Vec::<AnyElement>::new())
    };

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: sheet_test_id.map(|id| Arc::from(format!("{id}-drag-handle"))),
            ..Default::default()
        },
        move |cx| {
            vec![cx.container(wrapper, move |cx| {
                vec![cx.flex(row, move |_cx| vec![handle])]
            })]
        },
    )
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

fn absolute_fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.inset = fret_ui::element::InsetStyle {
        top: Some(Px(0.0)),
        right: Some(Px(0.0)),
        bottom: Some(Px(0.0)),
        left: Some(Px(0.0)),
    };
    layout
}
