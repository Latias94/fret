//! Material 3 bottom sheet primitives (P2).
//!
//! Outcome-oriented implementation:
//! - Token-driven styling via `md.comp.sheet.bottom.*` (Material Web v30).
//! - Modal variant uses `OverlayRequest::modal` with a scrim and focus trap/restore.
//! - Standard variant is a docked container surface (non-overlay), suitable for scaffold-like layouts.

use std::sync::Arc;
use std::sync::OnceLock;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{DismissReason, DismissRequestCx, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, FractionalRenderTransformProps, InsetEdge,
    InteractivityGateProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y,
    PressableProps, RingPlacement, RingStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};

use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::foundation::surface::material_surface_style;
use crate::foundation::test_id::{absolute_region_layout, diagnostic_anchor, part_test_id};
use crate::motion::{self, SpringAnimator};
use crate::tokens::sheet_bottom as sheet_tokens;

const BOTTOM_SHEET_PANE_LABEL: &str = "Bottom sheet";
const BOTTOM_SHEET_CLOSE_LABEL: &str = "Close sheet";
const BOTTOM_SHEET_DRAG_HANDLE_LABEL: &str = "Drag handle";

fn default_modal_bottom_sheet_test_id() -> Arc<str> {
    static ID: OnceLock<Arc<str>> = OnceLock::new();
    ID.get_or_init(|| Arc::<str>::from("material3-modal-bottom-sheet"))
        .clone()
}

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

impl Default for DockedBottomSheet {
    fn default() -> Self {
        Self::new()
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

    #[track_caller]
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
            column.gap = Px(0.0).into();
            column.layout.size.width = Length::Fill;

            // Compose baseline: `SheetMaxWidth = 640.dp`.
            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.max_width = Some(Length::Px(Px(640.0)));
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
                #[derive(Default)]
                struct DerivedDragHandleTestId {
                    base: Option<Arc<str>>,
                    drag_handle: Option<Arc<str>>,
                }

                let drag_handle_test_id = cx.slot_state(DerivedDragHandleTestId::default, |st| {
                    if st.base.as_deref() != test_id_for_children.as_deref() {
                        st.base = test_id_for_children.clone();
                        st.drag_handle = st.base.as_ref().map(|id| part_test_id(id, "drag-handle"));
                    }
                    st.drag_handle.clone()
                });

                let mut out: Vec<AnyElement> = Vec::new();
                if drag_handle {
                    out.push(drag_handle_element(cx, drag_handle_test_id.as_ref()));
                }
                out.extend(content(cx));
                out
            });

            let chrome_test_id = test_id.as_ref().map(|id| part_test_id(id, "chrome"));

            let semantics_role = if is_modal {
                SemanticsRole::Dialog
            } else {
                SemanticsRole::Group
            };
            let semantics_label = is_modal.then(|| Arc::<str>::from(BOTTOM_SHEET_PANE_LABEL));

            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: semantics_role,
                    label: semantics_label,
                    test_id,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(container, move |cx| {
                        let mut children = Vec::new();
                        if let Some(test_id) = chrome_test_id.clone() {
                            children.push(diagnostic_anchor(
                                cx,
                                test_id,
                                absolute_region_layout(
                                    InsetEdge::Px(Px(0.0)),
                                    InsetEdge::Px(Px(0.0)),
                                    Length::Fill,
                                    Length::Fill,
                                ),
                            ));
                        }
                        children.push(content_el);
                        children
                    })]
                },
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
            easing_key: None,
            on_dismiss_request: None,
            drag_handle: true,
            test_id: None,
        }
    }

    /// Creates a modal bottom sheet with a controlled/uncontrolled open model.
    ///
    /// When `open` is `None`, the sheet stores its internal open model at the root call site and
    /// initializes it from `default_open`.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = controllable_state::use_controllable_model(cx, open, || default_open).model();
        Self::new(open)
    }

    /// Default teaching-surface constructor for a sheet that owns its open model.
    pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {
        Self::new_controllable(cx, None, false)
    }

    /// Returns the resolved open model, including the internally owned model for uncontrolled use.
    pub fn open_model(&self) -> Model<bool> {
        self.open.clone()
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

    #[track_caller]
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

            let scrim_base = {
                let theme = Theme::global(&*cx.app);
                sheet_tokens::modal_scrim_color(theme)
            };

            let motion = drive_modal_bottom_sheet_motion(
                cx,
                open_now,
                open_duration_ms,
                close_duration_ms,
                easing_key.as_deref(),
            );
            let presence = OverlayPresence {
                present: motion.present,
                interactive: motion.interactive,
            };

            let underlay_el = underlay(cx);

            if presence.present {
                let scrim_opacity = {
                    let theme = Theme::global(&*cx.app);
                    sheet_tokens::modal_scrim_opacity(theme, scrim_opacity)
                };
                let scrim_alpha =
                    (scrim_base.a * scrim_opacity * motion.scrim_progress).clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_base, scrim_alpha);

                let dismiss_handler: OnDismissRequest = on_dismiss_request.unwrap_or_else(|| {
                    let open = open.clone();
                    Arc::new(move |host, action_cx, _cx: &mut DismissRequestCx| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.request_redraw(action_cx.window);
                    })
                });
                let dismiss_handler_for_request = dismiss_handler.clone();

                #[derive(Default)]
                struct DerivedTestIds {
                    base: Option<Arc<str>>,
                    scrim: Option<Arc<str>>,
                    scrim_chrome: Option<Arc<str>>,
                    sheet: Option<Arc<str>>,
                }

                let (scrim_test_id, scrim_chrome_test_id, sheet_test_id) =
                    cx.slot_state(DerivedTestIds::default, |st| {
                        if st.base.as_deref() != test_id.as_deref() {
                            st.base = test_id.clone();
                            st.scrim = st.base.as_ref().map(|id| part_test_id(id, "scrim"));
                            st.scrim_chrome =
                                st.scrim.as_ref().map(|id| part_test_id(id, "chrome"));
                            st.sheet = st.base.as_ref().map(|id| part_test_id(id, "sheet"));
                        }
                        (st.scrim.clone(), st.scrim_chrome.clone(), st.sheet.clone())
                    });

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
                                            label: Some(Arc::<str>::from(BOTTOM_SHEET_CLOSE_LABEL)),
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

                                        let mut chrome = cx.container(
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
                                        );
                                        if let Some(test_id) = scrim_chrome_test_id.clone() {
                                            chrome = chrome.test_id(test_id);
                                        }
                                        vec![chrome]
                                    },
                                )
                            });

                            let panel = cx.named("panel", |cx| {
                                let translate_y_fraction = 1.0 - motion.sheet_progress;

                                let mut align = FlexProps::default();
                                align.direction = Axis::Vertical;
                                align.justify = MainAlign::End;
                                align.align = CrossAlign::Center;
                                align.wrap = false;
                                align.layout.size.width = Length::Fill;
                                align.layout.size.height = Length::Fill;

                                let docked =
                                    DockedBottomSheet::new()
                                        .variant(DockedBottomSheetVariant::Modal)
                                        .drag_handle(drag_handle)
                                        .test_id(sheet_test_id.clone().unwrap_or_else(|| {
                                            default_modal_bottom_sheet_test_id()
                                        }));

                                let content_el = docked.into_element(cx, move |cx| content(cx));
                                let trapped =
                                    focus_scope_prim::focus_trap(cx, move |_cx| vec![content_el]);

                                let mut transform_layout = LayoutStyle::default();
                                transform_layout.size.width = Length::Fill;
                                let moving_sheet = cx.fractional_render_transform_props(
                                    FractionalRenderTransformProps {
                                        layout: transform_layout,
                                        translate_x_fraction: 0.0,
                                        translate_y_fraction,
                                    },
                                    move |_cx| vec![trapped],
                                );

                                let stacked = cx.flex(align, move |_cx| vec![moving_sheet]);

                                wrap_interactivity_gated(cx, presence.interactive, vec![stacked])
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

#[derive(Debug, Clone, Copy)]
struct ModalBottomSheetMotion {
    present: bool,
    interactive: bool,
    sheet_progress: f32,
    scrim_progress: f32,
}

#[track_caller]
fn drive_modal_bottom_sheet_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<&str>,
) -> ModalBottomSheetMotion {
    if open_duration_ms.is_some() || close_duration_ms.is_some() || easing_key.is_some() {
        let (default_duration_ms, bezier) = {
            let theme = Theme::global(&*cx.app);
            let default_duration_ms = theme
                .duration_ms_by_key("md.sys.motion.duration.medium2")
                .unwrap_or(300);
            let easing_key = easing_key.unwrap_or("md.sys.motion.easing.emphasized");
            let bezier = theme
                .easing_by_key(easing_key)
                .unwrap_or(fret_ui::theme::CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 1.0,
                });
            (default_duration_ms, bezier)
        };

        let open_ms = open_duration_ms.unwrap_or(default_duration_ms);
        let close_ms = close_duration_ms.unwrap_or(default_duration_ms);
        let transition = OverlayController::transition_with_durations_and_cubic_bezier(
            cx,
            open,
            motion::ms_to_frames(open_ms),
            motion::ms_to_frames(close_ms),
            bezier,
        );
        return ModalBottomSheetMotion {
            present: transition.present,
            interactive: open,
            sheet_progress: transition.progress,
            scrim_progress: transition.progress,
        };
    }

    #[derive(Default)]
    struct State {
        sheet: SpringAnimator,
        scrim: SpringAnimator,
    }

    let now_frame = cx.frame_id.0;
    let target = if open { 1.0 } else { 0.0 };
    let (sheet_spec, scrim_spec) = {
        let theme = Theme::global(&*cx.app);
        (
            sys_spring_in_scope(&*cx, theme, MotionSchemeKey::DefaultSpatial),
            sys_spring_in_scope(&*cx, theme, MotionSchemeKey::DefaultEffects),
        )
    };

    let (sheet_progress, scrim_progress, animating) = cx.slot_state(State::default, |st| {
        if !st.sheet.is_initialized() {
            st.sheet.reset(now_frame, target);
        }
        if !st.scrim.is_initialized() {
            st.scrim.reset(now_frame, target);
        }

        st.sheet.set_target(now_frame, target, sheet_spec);
        st.scrim.set_target(now_frame, target, scrim_spec);
        st.sheet.advance(now_frame);
        st.scrim.advance(now_frame);

        (
            st.sheet.value(),
            st.scrim.value(),
            st.sheet.is_active() || st.scrim.is_active(),
        )
    });

    if animating {
        cx.request_frame();
    }

    let present = open || animating || sheet_progress > 0.001 || scrim_progress > 0.001;
    ModalBottomSheetMotion {
        present,
        interactive: open,
        sheet_progress,
        scrim_progress,
    }
}

fn fullscreen_motion_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn wrap_interactivity_gated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    interactive: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    let layout = fullscreen_motion_layout();
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout,
            present: true,
            interactive,
        },
        move |_cx| children,
    )
}

fn drag_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    drag_handle_test_id: Option<&Arc<str>>,
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
    }
    .into();

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
            label: Some(Arc::<str>::from(BOTTOM_SHEET_DRAG_HANDLE_LABEL)),
            test_id: drag_handle_test_id.cloned(),
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
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };
    layout
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::elements::with_element_cx;
    use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

    use super::*;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn modal_bottom_sheet_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(true);

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-bottom-sheet-controlled",
            |cx| {
                let sheet = ModalBottomSheet::new_controllable(cx, Some(controlled.clone()), false);
                assert_eq!(sheet.open_model(), controlled);
            },
        );
    }

    #[test]
    fn modal_bottom_sheet_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-bottom-sheet-default-open",
            |cx| {
                let sheet = ModalBottomSheet::new_controllable(cx, None, true);
                let open = cx
                    .watch_model(&sheet.open_model())
                    .layout()
                    .copied()
                    .unwrap_or(false);
                assert!(open);
            },
        );
    }

    #[test]
    fn modal_bottom_sheet_uncontrolled_multiple_instances_do_not_share_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "material3-bottom-sheet-uncontrolled-scope",
            |cx| {
                let a = ModalBottomSheet::uncontrolled(cx);
                let b = ModalBottomSheet::uncontrolled(cx);
                assert_ne!(a.open_model(), b.open_model());
            },
        );
    }
}
