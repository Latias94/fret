//! Material 3 time picker primitives (P2).
//!
//! Outcome-oriented implementation:
//! - Token-driven time selector + clock dial outcomes via `md.comp.time-picker.*`.
//! - Modal variant uses `OverlayRequest::modal` with a scrim and focus trap/restore.
//! - Selection is staged while open and applied on confirm.

use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, CursorIcon, Edges, KeyCode, MouseButton, Px, SemanticsRole, TextOverflow,
    TextWrap,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, DismissReason, DismissRequestCx, OnActivate, OnDismissRequest, PointerDownCx,
    PointerMoveCx, PointerUpCx, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, HoverRegionProps, LayoutStyle, Length,
    MainAlign, Overflow, PointerRegionProps, PressableA11y, PressableProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};
use time::Time;

use crate::button::{Button, ButtonVariant};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::interaction::{PressableInteraction, pressable_interaction};
use crate::foundation::surface::material_surface_style;
use crate::icon_button::{IconButton, IconButtonVariant};
use crate::motion;
use crate::tokens::time_input as time_input_tokens;
use crate::tokens::time_picker as time_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePickerVariant {
    #[default]
    Docked,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePickerDisplayMode {
    #[default]
    Dial,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TimePickerSelection {
    #[default]
    Hour,
    Minute,
}

#[derive(Default)]
struct DockedRuntime {
    selection: Option<Model<TimePickerSelection>>,
    display_mode: Option<Model<TimePickerDisplayMode>>,
    input_hour: Option<Model<String>>,
    input_minute: Option<Model<String>>,
    dial_dragging: Option<Model<bool>>,
    time_input_edit: Option<Model<TimeInputEditState>>,
}

#[derive(Clone)]
pub struct DockedTimePicker {
    time: Model<Time>,
    is_24h: bool,
    display_mode: TimePickerDisplayMode,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DockedTimePicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockedTimePicker")
            .field("time", &"<model>")
            .field("is_24h", &self.is_24h)
            .field("display_mode", &self.display_mode)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl DockedTimePicker {
    pub fn new(time: Model<Time>) -> Self {
        Self {
            time,
            is_24h: false,
            display_mode: TimePickerDisplayMode::Dial,
            test_id: None,
        }
    }

    pub fn is_24h(mut self, is_24h: bool) -> Self {
        self.is_24h = is_24h;
        self
    }

    pub fn display_mode(mut self, mode: TimePickerDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let selection_model = ensure_selection_model(cx, DockedRuntime::default);
            let selection = cx
                .get_model_copied(&selection_model, Invalidation::Layout)
                .unwrap_or_default();
            let display_mode_model =
                ensure_display_mode_model(cx, DockedRuntime::default, self.display_mode);
            let display_mode = cx
                .get_model_copied(&display_mode_model, Invalidation::Layout)
                .unwrap_or_default();
            let time_now = cx
                .get_model_copied(&self.time, Invalidation::Layout)
                .unwrap_or_else(default_time);
            let (input_hour, input_minute) =
                ensure_time_input_models(cx, DockedRuntime::default, time_now, self.is_24h);
            let dial_dragging_model = ensure_dial_dragging_model(cx, DockedRuntime::default);
            let time_input_edit = ensure_time_input_edit_state_model(cx, DockedRuntime::default);

            // Compose baseline: TimePickerMaxHeight (384dp) + padding.
            // We keep a stable box to make headless suites deterministic.
            let width = Px(368.0);
            let height = Px(384.0);

            let (surface, corner_radii) = {
                let theme = Theme::global(&*cx.app);
                let background = time_tokens::container_color(theme);
                let elevation = time_tokens::container_elevation(theme);
                let corner_radii = time_tokens::container_shape(theme);
                let surface =
                    material_surface_style(theme, background, elevation, None, corner_radii);
                (surface, corner_radii)
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(width);
            layout.size.height = Length::Px(height);
            layout.overflow = Overflow::Clip;

            let mut container = ContainerProps::default();
            container.layout = layout;
            container.background = Some(surface.background);
            container.shadow = surface.shadow;
            container.corner_radii = corner_radii;

            let content = time_picker_contents(
                cx,
                time_now,
                self.time.clone(),
                selection_model.clone(),
                selection,
                display_mode_model.clone(),
                display_mode,
                dial_dragging_model.clone(),
                time_input_edit.clone(),
                input_hour.clone(),
                input_minute.clone(),
                self.is_24h,
                None,
            );

            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: self.test_id.clone(),
                    ..Default::default()
                },
                move |cx| vec![cx.container(container, move |_cx| vec![content])],
            )
        })
    }
}

#[derive(Default)]
struct DialogRuntime {
    models: Option<DialogModels>,
    was_open: bool,
}

#[derive(Clone)]
struct DialogModels {
    draft_time: Model<Time>,
    selection: Model<TimePickerSelection>,
    display_mode: Model<TimePickerDisplayMode>,
    input_hour: Model<String>,
    input_minute: Model<String>,
    dial_dragging: Model<bool>,
    time_input_edit: Model<TimeInputEditState>,
}

#[derive(Clone)]
pub struct TimePickerDialog {
    open: Model<bool>,
    selected: Model<Time>,
    is_24h: bool,
    initial_display_mode: TimePickerDisplayMode,
    scrim_opacity: f32,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for TimePickerDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimePickerDialog")
            .field("open", &"<model>")
            .field("selected", &"<model>")
            .field("is_24h", &self.is_24h)
            .field("initial_display_mode", &self.initial_display_mode)
            .field("scrim_opacity", &self.scrim_opacity)
            .field("open_duration_ms", &self.open_duration_ms)
            .field("close_duration_ms", &self.close_duration_ms)
            .field("easing_key", &self.easing_key)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl TimePickerDialog {
    pub fn new(open: Model<bool>, selected: Model<Time>) -> Self {
        Self {
            open,
            selected,
            is_24h: false,
            initial_display_mode: TimePickerDisplayMode::Dial,
            // Align with Dialog defaults.
            scrim_opacity: 0.32,
            open_duration_ms: None,
            close_duration_ms: None,
            easing_key: Some(Arc::<str>::from("md.sys.motion.easing.emphasized")),
            on_dismiss_request: None,
            test_id: None,
        }
    }

    pub fn is_24h(mut self, is_24h: bool) -> Self {
        self.is_24h = is_24h;
        self
    }

    pub fn initial_display_mode(mut self, mode: TimePickerDisplayMode) -> Self {
        self.initial_display_mode = mode;
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
    ) -> AnyElement {
        cx.scope(|cx| {
            let open_now = cx
                .get_model_copied(&self.open, Invalidation::Layout)
                .unwrap_or(false);

            let prev_open = cx.with_state(DialogRuntime::default, |st| st.was_open);
            cx.with_state(DialogRuntime::default, |st| st.was_open = open_now);

            let existing = cx.with_state(DialogRuntime::default, |st| st.models.clone());
            let models = match existing {
                Some(models) => models,
                None => {
                    let draft_time = cx.app.models_mut().insert(default_time());
                    let selection = cx.app.models_mut().insert(TimePickerSelection::Hour);
                    let display_mode = cx
                        .app
                        .models_mut()
                        .insert(self.initial_display_mode);
                    let (hour, minute) = time_to_display(default_time(), self.is_24h);
                    let input_hour = cx.app.models_mut().insert(format!("{hour:02}"));
                    let input_minute = cx.app.models_mut().insert(format!("{minute:02}"));
                    let dial_dragging = cx.app.models_mut().insert(false);
                    let time_input_edit = cx.app.models_mut().insert(TimeInputEditState::default());
                    let models = DialogModels {
                        draft_time,
                        selection,
                        display_mode,
                        input_hour,
                        input_minute,
                        dial_dragging,
                        time_input_edit,
                    };
                    cx.with_state(DialogRuntime::default, |st| st.models = Some(models.clone()));
                    models
                }
            };

            if open_now && !prev_open {
                let external = cx
                    .get_model_copied(&self.selected, Invalidation::Layout)
                    .unwrap_or_else(default_time);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.draft_time, |t| *t = external);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.selection, |s| *s = TimePickerSelection::Hour);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.dial_dragging, |v| *v = false);
                let _ = cx.app.models_mut().update(&models.time_input_edit, |st| {
                    *st = TimeInputEditState::default();
                });
                let _ = cx.app.models_mut().update(&models.display_mode, |m| {
                    *m = self.initial_display_mode;
                });
                let (hour, minute) = time_to_display(external, self.is_24h);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.input_hour, |v| *v = format!("{hour:02}"));
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.input_minute, |v| *v = format!("{minute:02}"));
            }

            let easing_key = self
                .easing_key
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));

            let (open_ms, close_ms, bezier, scrim_base) = {
                let theme = Theme::global(&*cx.app);

                let open_ms = self
                    .open_duration_ms
                    .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                    .unwrap_or(300);
                let close_ms = self
                    .close_duration_ms
                    .or_else(|| theme.duration_ms_by_key("md.sys.motion.duration.medium2"))
                    .unwrap_or(300);

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

                (open_ms, close_ms, bezier, scrim_base)
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

            let underlay_el = underlay(cx);

            if presence.present {
                let open_model_for_request = self.open.clone();
                let open_model_for_overlay = self.open.clone();

                let scrim_alpha = (scrim_base.a * self.scrim_opacity * transition.progress)
                    .clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_base, scrim_alpha);

                let dismiss_handler: OnDismissRequest =
                    self.on_dismiss_request.clone().unwrap_or_else(|| {
                        let open = open_model_for_request.clone();
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
                    panel: Option<Arc<str>>,
                }

                let (scrim_test_id, panel_test_id) =
                    cx.with_state(DerivedTestIds::default, |st| {
                        if st.base.as_deref() != self.test_id.as_deref() {
                            st.base = self.test_id.clone();
                            st.scrim = st.base.as_ref().map(|id| {
                                Arc::from(format!("{}-scrim", id.as_ref()))
                            });
                            st.panel = st.base.as_ref().map(|id| {
                                Arc::from(format!("{}-panel", id.as_ref()))
                            });
                        }
                        (st.scrim.clone(), st.panel.clone())
                    });

                let overlay_root = cx.named("time_picker_overlay_root", |cx| {
                    cx.focus_scope(
                        fret_ui_kit::primitives::focus_scope::FocusScopeProps::default(),
                        move |cx| {
                            let mut align = FlexProps::default();
                            align.direction = Axis::Vertical;
                            align.justify = MainAlign::Center;
                            align.align = CrossAlign::Center;
                            align.wrap = false;

                            let scrim = cx.named("scrim", |cx| {
                                cx.pressable(
                                    PressableProps {
                                        enabled: open_now,
                                        focusable: false,
                                        a11y: PressableA11y {
                                            role: Some(SemanticsRole::Button),
                                            label: Some(Arc::<str>::from("Dismiss")),
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
                                                        DismissReason::OutsidePress {
                                                            pointer: None,
                                                        },
                                                    );
                                                    dismiss_handler(host, action_cx, &mut dismiss_cx);
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

                            let open = open_model_for_overlay.clone();
                            let on_cancel: OnActivate = Arc::new(move |host, action_cx, _reason| {
                                let _ = host.models_mut().update(&open, |v| *v = false);
                                host.request_redraw(action_cx.window);
                            });
                            let open = open_model_for_overlay.clone();
                            let selected = self.selected.clone();
                            let draft = models.draft_time.clone();
                            let on_confirm: OnActivate = Arc::new(move |host, action_cx, _reason| {
                                let draft_now = host
                                    .models_mut()
                                    .get_cloned(&draft)
                                    .unwrap_or_else(default_time);
                                let _ = host.models_mut().update(&selected, |v| *v = draft_now);
                                let _ = host.models_mut().update(&open, |v| *v = false);
                                host.request_redraw(action_cx.window);
                            });

                            let picker = cx.named("panel", move |cx| {
                                let selection = cx
                                    .get_model_copied(&models.selection, Invalidation::Layout)
                                    .unwrap_or_default();
                                let display_mode = cx
                                    .get_model_copied(&models.display_mode, Invalidation::Layout)
                                    .unwrap_or_default();
                                let time_now = cx
                                    .get_model_copied(&models.draft_time, Invalidation::Layout)
                                    .unwrap_or_else(default_time);

                                time_picker_modal_panel(
                                    cx,
                                    time_now,
                                    models.draft_time.clone(),
                                    models.selection.clone(),
                                    selection,
                                    models.display_mode.clone(),
                                    display_mode,
                                    models.dial_dragging.clone(),
                                    models.time_input_edit.clone(),
                                    models.input_hour.clone(),
                                    models.input_minute.clone(),
                                    self.is_24h,
                                    panel_test_id.clone(),
                                    on_cancel.clone(),
                                    on_confirm.clone(),
                                )
                            });

                            let trapped = focus_scope_prim::focus_trap(cx, move |_cx| vec![picker]);
                            let stacked = cx.flex(align, move |_cx| vec![trapped]);

                            let opacity = transition.progress;
                            let scale = 0.95 + 0.05 * transition.progress;
                            let transform = fret_core::Transform2D::scale_uniform(scale);
                            let panel = fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                                cx,
                                opacity,
                                transform,
                                presence.interactive,
                                vec![stacked],
                            );

                            vec![scrim, panel]
                        },
                    )
                });

                let overlay_id = cx.root_id();
                let mut request = overlay_controller::OverlayRequest::modal(
                    overlay_id,
                    None,
                    open_model_for_request.clone(),
                    presence,
                    vec![overlay_root],
                );
                request.root_name = Some(format!("material3.time_picker_dialog.{}", overlay_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = Some(dismiss_handler_for_request);
                OverlayController::request(cx, request);
            }

            underlay_el
        })
    }
}

fn time_picker_modal_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    selection: TimePickerSelection,
    display_mode_model: Model<TimePickerDisplayMode>,
    display_mode: TimePickerDisplayMode,
    dial_dragging_model: Model<bool>,
    time_input_edit: Model<TimeInputEditState>,
    input_hour: Model<String>,
    input_minute: Model<String>,
    is_24h: bool,
    test_id: Option<Arc<str>>,
    on_cancel: OnActivate,
    on_confirm: OnActivate,
) -> AnyElement {
    // Compose baseline: TimePickerMaxHeight (384dp) + content padding.
    let width = Px(368.0);
    let height = Px(384.0);

    let (surface, corner_radii) = {
        let theme = Theme::global(&*cx.app);
        let background = time_tokens::container_color(theme);
        let elevation = time_tokens::container_elevation(theme);
        let corner_radii = time_tokens::container_shape(theme);
        let surface = material_surface_style(theme, background, elevation, None, corner_radii);
        (surface, corner_radii)
    };

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(width);
    layout.size.height = Length::Px(height);
    layout.overflow = Overflow::Clip;

    let mut container = ContainerProps::default();
    container.layout = layout;
    container.background = Some(surface.background);
    container.shadow = surface.shadow;
    container.corner_radii = corner_radii;

    let content = time_picker_contents(
        cx,
        time_now,
        time_model,
        selection_model,
        selection,
        display_mode_model,
        display_mode,
        dial_dragging_model,
        time_input_edit,
        input_hour,
        input_minute,
        is_24h,
        Some((on_cancel, on_confirm)),
    );

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Dialog,
            test_id,
            ..Default::default()
        },
        move |cx| vec![cx.container(container, move |_cx| vec![content])],
    )
}

fn time_picker_contents<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    selection: TimePickerSelection,
    display_mode_model: Model<TimePickerDisplayMode>,
    display_mode: TimePickerDisplayMode,
    dial_dragging_model: Model<bool>,
    time_input_edit: Model<TimeInputEditState>,
    input_hour: Model<String>,
    input_minute: Model<String>,
    is_24h: bool,
    actions: Option<(OnActivate, OnActivate)>,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.direction = Axis::Vertical;
    props.justify = MainAlign::Start;
    props.align = CrossAlign::Stretch;
    props.wrap = false;
    props.gap = Px(16.0);
    props.padding = Edges::all(Px(24.0));
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;

    cx.flex(props, move |cx| {
        let title_text = {
            let (style, color) = {
                let theme = Theme::global(&*cx.app);
                (
                    time_tokens::headline_style(theme),
                    time_tokens::headline_color(theme),
                )
            };
            let mut t = TextProps::new(Arc::<str>::from("Select time"));
            t.style = Some(style);
            t.color = Some(color);
            t.wrap = TextWrap::None;
            t.overflow = TextOverflow::Ellipsis;
            cx.text_props(t)
        };

        let toggle_icon = match display_mode {
            TimePickerDisplayMode::Dial => IconId::new("lucide.keyboard"),
            TimePickerDisplayMode::Input => IconId::new("lucide.clock"),
        };
        let toggle_label = match display_mode {
            TimePickerDisplayMode::Dial => "Switch to input",
            TimePickerDisplayMode::Input => "Switch to dial",
        };
        let toggle_test_id = "time-picker-mode-toggle";
        let on_toggle: OnActivate = Arc::new({
            let display_mode_model = display_mode_model.clone();
            let selection_model = selection_model.clone();
            let time_model = time_model.clone();
            let input_hour = input_hour.clone();
            let input_minute = input_minute.clone();
            let time_input_edit = time_input_edit.clone();
            move |host, action_cx, _reason| {
                let next = match host.models_mut().get_cloned(&display_mode_model) {
                    Some(TimePickerDisplayMode::Input) => TimePickerDisplayMode::Dial,
                    _ => TimePickerDisplayMode::Input,
                };
                let _ = host.models_mut().update(&display_mode_model, |m| *m = next);

                if next == TimePickerDisplayMode::Input {
                    let time_now = host
                        .models_mut()
                        .get_cloned(&time_model)
                        .unwrap_or_else(default_time);
                    let (hour, minute) = time_to_display(time_now, is_24h);
                    let _ = host
                        .models_mut()
                        .update(&input_hour, |v| *v = format!("{hour:02}"));
                    let _ = host
                        .models_mut()
                        .update(&input_minute, |v| *v = format!("{minute:02}"));
                    let _ = host
                        .models_mut()
                        .update(&selection_model, |s| *s = TimePickerSelection::Hour);
                    let _ = host.models_mut().update(&time_input_edit, |st| {
                        *st = TimeInputEditState::default();
                    });
                }

                host.request_redraw(action_cx.window);
            }
        });

        let toggle = IconButton::new(toggle_icon)
            .variant(IconButtonVariant::Standard)
            .a11y_label(toggle_label)
            .test_id(toggle_test_id)
            .on_activate(on_toggle)
            .into_element(cx);

        let title_row = cx.flex(
            FlexProps {
                direction: Axis::Horizontal,
                justify: MainAlign::SpaceBetween,
                align: CrossAlign::Center,
                wrap: false,
                layout: {
                    let mut l = LayoutStyle::default();
                    l.size.width = Length::Fill;
                    l
                },
                ..Default::default()
            },
            move |_cx| vec![title_text, toggle],
        );

        let mut out = vec![title_row];

        match display_mode {
            TimePickerDisplayMode::Dial => {
                let display = time_picker_display(
                    cx,
                    time_now,
                    time_model.clone(),
                    selection_model.clone(),
                    selection,
                    is_24h,
                );

                let dial = time_picker_clock_dial(
                    cx,
                    time_now,
                    time_model.clone(),
                    selection_model.clone(),
                    selection,
                    dial_dragging_model.clone(),
                    is_24h,
                );

                let period = (!is_24h)
                    .then(|| time_picker_period_selector(cx, time_now, time_model.clone()));

                let dial_and_period = cx.flex(
                    FlexProps {
                        direction: Axis::Horizontal,
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                        gap: Px(12.0),
                        ..Default::default()
                    },
                    move |_cx| {
                        let mut out = vec![dial];
                        if let Some(p) = period {
                            out.push(p);
                        }
                        out
                    },
                );

                out.push(display);
                out.push(dial_and_period);
            }
            TimePickerDisplayMode::Input => {
                out.push(time_picker_time_input(
                    cx,
                    time_now,
                    time_model.clone(),
                    time_input_edit.clone(),
                    input_hour.clone(),
                    input_minute.clone(),
                    is_24h,
                ));
            }
        }

        if let Some((on_cancel, on_confirm)) = actions.clone() {
            out.push(time_picker_actions(cx, on_cancel, on_confirm));
        }

        out
    })
}

fn time_picker_display<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    selection: TimePickerSelection,
    is_24h: bool,
) -> AnyElement {
    let (hour, minute) = time_to_display(time_now, is_24h);
    let hour_s = Arc::<str>::from(format!("{hour:02}"));
    let minute_s = Arc::<str>::from(format!("{minute:02}"));

    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.justify = MainAlign::Start;
    props.align = CrossAlign::Center;
    props.wrap = false;
    props.gap = Px(8.0);

    cx.flex(props, move |cx| {
        let hour_el = time_selector_field(
            cx,
            Arc::<str>::from("Hour"),
            hour_s.clone(),
            selection == TimePickerSelection::Hour,
            "time-picker-hour-selector",
            time_model.clone(),
            selection_model.clone(),
            TimePickerSelection::Hour,
        );

        let (sep_style, sep_color) = {
            let theme = Theme::global(&*cx.app);
            (
                time_tokens::time_selector_separator_style(theme),
                time_tokens::time_selector_separator_color(theme),
            )
        };
        let mut sep = TextProps::new(Arc::<str>::from(":"));
        sep.style = Some(sep_style);
        sep.color = Some(sep_color);
        sep.wrap = TextWrap::None;
        sep.overflow = TextOverflow::Clip;
        let sep_el = cx.text_props(sep);

        let minute_el = time_selector_field(
            cx,
            Arc::<str>::from("Minute"),
            minute_s.clone(),
            selection == TimePickerSelection::Minute,
            "time-picker-minute-selector",
            time_model.clone(),
            selection_model.clone(),
            TimePickerSelection::Minute,
        );

        vec![hour_el, sep_el, minute_el]
    })
}

fn time_picker_time_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    time_input_edit: Model<TimeInputEditState>,
    input_hour: Model<String>,
    input_minute: Model<String>,
    is_24h: bool,
) -> AnyElement {
    apply_time_input_models(
        cx,
        time_now,
        time_model.clone(),
        input_hour.clone(),
        input_minute.clone(),
        is_24h,
    );

    let hour_column = time_input_field_column(
        cx,
        Arc::<str>::from("Hour"),
        Arc::<str>::from("Hour"),
        TimeInputFieldKind::Hour,
        time_now,
        time_model.clone(),
        time_input_edit.clone(),
        input_hour,
        "time-input-hour",
        is_24h,
    );
    let minute_column = time_input_field_column(
        cx,
        Arc::<str>::from("Minute"),
        Arc::<str>::from("Minute"),
        TimeInputFieldKind::Minute,
        time_now,
        time_model.clone(),
        time_input_edit.clone(),
        input_minute,
        "time-input-minute",
        is_24h,
    );

    let (sep_style, sep_color) = {
        let theme = Theme::global(&*cx.app);
        (
            time_input_tokens::time_input_field_separator_style(theme),
            time_input_tokens::time_input_field_separator_color(theme),
        )
    };
    let mut sep = TextProps::new(Arc::<str>::from(":"));
    sep.style = Some(sep_style);
    sep.color = Some(sep_color);
    sep.wrap = TextWrap::None;
    sep.overflow = TextOverflow::Clip;
    let sep_el = cx.text_props(sep);

    let period = (!is_24h).then(|| time_input_period_selector(cx, time_now, time_model.clone()));

    let mut row = FlexProps::default();
    row.direction = Axis::Horizontal;
    row.justify = MainAlign::Start;
    row.align = CrossAlign::Center;
    row.wrap = false;
    row.gap = Px(8.0);

    cx.flex(row, move |_cx| {
        let mut out = vec![hour_column, sep_el, minute_column];
        if let Some(p) = period {
            out.push(p);
        }
        out
    })
}

fn time_input_field_column<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    a11y_label: Arc<str>,
    supporting_text: Arc<str>,
    kind: TimeInputFieldKind,
    time_now: Time,
    time_model: Model<Time>,
    time_input_edit: Model<TimeInputEditState>,
    model: Model<String>,
    test_id: &'static str,
    is_24h: bool,
) -> AnyElement {
    let field = time_input_field(
        cx,
        a11y_label,
        kind,
        time_now,
        time_model,
        time_input_edit,
        model,
        test_id,
        is_24h,
    );

    let (supporting_style, supporting_color) = {
        let theme = Theme::global(&*cx.app);
        (
            time_input_tokens::time_input_field_supporting_text_style(theme),
            time_input_tokens::time_input_field_supporting_text_color(theme),
        )
    };
    let mut supporting = TextProps::new(supporting_text);
    supporting.style = Some(supporting_style);
    supporting.color = Some(supporting_color);
    supporting.wrap = TextWrap::None;
    supporting.overflow = TextOverflow::Clip;
    let supporting = cx.text_props(supporting);

    let mut col = FlexProps::default();
    col.direction = Axis::Vertical;
    col.justify = MainAlign::Start;
    col.align = CrossAlign::Center;
    col.wrap = false;
    col.gap = Px(4.0);
    cx.flex(col, move |_cx| vec![field, supporting])
}

fn time_input_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    a11y_label: Arc<str>,
    kind: TimeInputFieldKind,
    time_now: Time,
    time_model: Model<Time>,
    time_input_edit: Model<TimeInputEditState>,
    model: Model<String>,
    test_id: &'static str,
    is_24h: bool,
) -> AnyElement {
    let (width, height, corner_radii, focus_ring) = {
        let theme = Theme::global(&*cx.app);
        let width = time_input_tokens::time_input_field_container_width(theme);
        let height = time_input_tokens::time_input_field_container_height(theme);
        let corner_radii = time_input_tokens::time_input_field_container_shape(theme);
        let focus_ring = material_focus_ring_for_component(
            theme,
            time_input_tokens::COMPONENT_PREFIX,
            corner_radii,
        );
        (width, height, corner_radii, focus_ring)
    };

    let mut hover_layout = LayoutStyle::default();
    hover_layout.size.width = Length::Px(width);
    hover_layout.overflow = Overflow::Visible;
    let hover = HoverRegionProps {
        layout: hover_layout,
    };

    cx.hover_region(hover, move |cx, hovered| {
        let focused_out: Cell<bool> = Cell::new(false);

        let field = cx.pressable_with_id_props(|cx, st, pressable_id| {
            let enabled = true;
            let focused = enabled && st.focused;
            focused_out.set(focused);

            update_time_input_edit_ids(cx, &time_input_edit, kind, pressable_id);

            let (_gained_focus, lost_focus) =
                cx.with_state_for(pressable_id, TimeInputFieldRuntime::default, |rt| {
                    let gained = !rt.was_focused && focused;
                    let lost = rt.was_focused && !focused;
                    rt.was_focused = focused;
                    (gained, lost)
                });

            if lost_focus {
                normalize_time_input_on_blur(
                    cx,
                    time_now,
                    time_model.clone(),
                    model.clone(),
                    kind,
                    is_24h,
                );
            }

            cx.key_add_on_key_down_for(
                pressable_id,
                Arc::new({
                    let input_on_key = model.clone();
                    let edit_on_key = time_input_edit.clone();
                    move |host, acx, down| {
                        if down.repeat {
                            return false;
                        }
                        if down.modifiers.alt || down.modifiers.ctrl || down.modifiers.meta {
                            return false;
                        }

                        if down.key == KeyCode::Backspace {
                            let cur = host
                                .models_mut()
                                .get_cloned(&input_on_key)
                                .unwrap_or_default();
                            let mut cur = sanitize_time_input_digits(&cur);
                            if cur.is_empty() {
                                if kind == TimeInputFieldKind::Minute {
                                    let target = host
                                        .models_mut()
                                        .read(&edit_on_key, |st| st.hour_input)
                                        .ok()
                                        .flatten();
                                    if let Some(target) = target {
                                        host.request_focus(target);
                                        host.request_redraw(acx.window);
                                        return true;
                                    }
                                }
                                return false;
                            }

                            cur.pop();
                            let _ = host
                                .models_mut()
                                .update(&input_on_key, |v| *v = cur.clone());
                            host.request_redraw(acx.window);
                            return true;
                        }

                        let Some(digit) = keycode_digit(down.key) else {
                            return false;
                        };
                        let ch = char::from(b'0' + digit);

                        let cur = host
                            .models_mut()
                            .get_cloned(&input_on_key)
                            .unwrap_or_default();
                        let cur = sanitize_time_input_digits(&cur);
                        let mut next = if cur.len() >= 2 { String::new() } else { cur };
                        next.push(ch);
                        let _ = host
                            .models_mut()
                            .update(&input_on_key, |v| *v = next.clone());

                        if kind == TimeInputFieldKind::Hour && next.len() == 2 {
                            let target = host
                                .models_mut()
                                .read(&edit_on_key, |st| st.minute_input)
                                .ok()
                                .flatten();
                            if let Some(target) = target {
                                host.request_focus(target);
                            }
                        }

                        host.request_redraw(acx.window);
                        true
                    }
                }),
            );

            let (label_color, label_style) = {
                let theme = Theme::global(&*cx.app);
                (
                    time_input_tokens::time_input_field_label_color(theme, focused, hovered),
                    time_input_tokens::time_input_field_label_text_style(theme),
                )
            };
            let raw = cx
                .get_model_cloned(&model, Invalidation::Layout)
                .unwrap_or_default();
            let raw = sanitize_time_input_digits(&raw);

            let text = Arc::<str>::from(raw);
            let mut tp = TextProps::new(text);
            tp.style = Some(label_style);
            tp.color = Some(label_color);
            tp.wrap = TextWrap::None;
            tp.overflow = TextOverflow::Clip;
            let label = cx.text_props(tp);

            let mut center = FlexProps::default();
            center.direction = Axis::Horizontal;
            center.justify = MainAlign::Center;
            center.align = CrossAlign::Center;
            center.wrap = false;
            center.layout.size.width = Length::Fill;
            center.layout.size.height = Length::Fill;
            let content = cx.flex(center, move |_cx| vec![label]);

            let pressable_props = PressableProps {
                enabled,
                focusable: enabled,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::TextField),
                    label: Some(a11y_label.clone()),
                    test_id: Some(Arc::<str>::from(test_id)),
                    ..Default::default()
                },
                layout: {
                    let mut l = LayoutStyle::default();
                    l.overflow = Overflow::Visible;
                    l.size.width = Length::Fill;
                    l.size.height = Length::Fill;
                    l
                },
                focus_ring: Some(focus_ring),
                focus_ring_bounds: None,
            };
            (pressable_props, vec![content])
        });

        let focused = focused_out.get();

        let mut container = ContainerProps::default();
        container.layout.size.width = Length::Px(width);
        container.layout.size.height = Length::Px(height);
        container.layout.overflow = Overflow::Clip;
        container.corner_radii = corner_radii;
        container.background = Some({
            let theme = Theme::global(&*cx.app);
            time_input_tokens::time_input_field_container_color(theme, focused)
        });
        if focused {
            let (w, c) = {
                let theme = Theme::global(&*cx.app);
                (
                    time_input_tokens::time_input_field_focus_outline_width(theme),
                    time_input_tokens::time_input_field_focus_outline_color(theme),
                )
            };
            container.border = Edges::all(w);
            container.border_color = Some(c);
        }

        let overlay = (hovered && !focused).then(|| {
            let mut layout = LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(0.0));
            layout.inset.right = Some(Px(0.0));
            layout.inset.bottom = Some(Px(0.0));
            layout.inset.left = Some(Px(0.0));

            let c = {
                let theme = Theme::global(&*cx.app);
                let mut c = time_input_tokens::time_input_field_state_layer_color(theme);
                c.a = (c.a * time_input_tokens::time_input_field_state_layer_opacity(theme))
                    .clamp(0.0, 1.0);
                c
            };

            let mut overlay = ContainerProps::default();
            overlay.layout = layout;
            overlay.background = Some(c);
            overlay.corner_radii = corner_radii;
            cx.container(overlay, |_cx| Vec::new())
        });

        vec![match overlay {
            Some(overlay) => cx.container(container, move |_cx| vec![overlay, field]),
            None => cx.container(container, move |_cx| vec![field]),
        }]
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeInputFieldKind {
    Hour,
    Minute,
}

#[derive(Default)]
struct TimeInputFieldRuntime {
    was_focused: bool,
}

#[derive(Debug, Clone, Default)]
struct TimeInputEditState {
    hour_input: Option<GlobalElementId>,
    minute_input: Option<GlobalElementId>,
}

fn update_time_input_edit_ids<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    edit: &Model<TimeInputEditState>,
    kind: TimeInputFieldKind,
    id: GlobalElementId,
) {
    let cur = cx
        .get_model_cloned(edit, Invalidation::Layout)
        .unwrap_or_default();
    let changed = match kind {
        TimeInputFieldKind::Hour => cur.hour_input != Some(id),
        TimeInputFieldKind::Minute => cur.minute_input != Some(id),
    };
    if !changed {
        return;
    }
    let _ = cx.app.models_mut().update(edit, |st| match kind {
        TimeInputFieldKind::Hour => st.hour_input = Some(id),
        TimeInputFieldKind::Minute => st.minute_input = Some(id),
    });
}

fn normalize_time_input_on_blur<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    fallback_time: Time,
    time_model: Model<Time>,
    input_model: Model<String>,
    kind: TimeInputFieldKind,
    is_24h: bool,
) {
    let time_now = cx
        .get_model_copied(&time_model, Invalidation::Layout)
        .unwrap_or(fallback_time);
    let (hour, minute) = time_to_display(time_now, is_24h);
    let next = match kind {
        TimeInputFieldKind::Hour => format!("{hour:02}"),
        TimeInputFieldKind::Minute => format!("{minute:02}"),
    };
    let cur = cx
        .get_model_cloned(&input_model, Invalidation::Layout)
        .unwrap_or_default();
    if cur == next {
        return;
    }
    let _ = cx.app.models_mut().update(&input_model, |v| *v = next);
}

fn keycode_digit(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::Digit0 | KeyCode::Numpad0 => Some(0),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(1),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(2),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some(3),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some(4),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(5),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some(6),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some(7),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some(8),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some(9),
        _ => None,
    }
}

fn time_selector_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    a11y_label: Arc<str>,
    text: Arc<str>,
    selected: bool,
    test_id: &'static str,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    selection_kind: TimePickerSelection,
) -> AnyElement {
    let (corner_radii, container_w, container_h, focus_ring) = {
        let theme = Theme::global(&*cx.app);
        let corner_radii = time_tokens::time_selector_shape(theme);
        let container_w = time_tokens::time_selector_container_width(theme);
        let container_h = time_tokens::time_selector_container_height(theme);
        let focus_ring =
            material_focus_ring_for_component(theme, time_tokens::COMPONENT_PREFIX, corner_radii);
        (corner_radii, container_w, container_h, focus_ring)
    };

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = true;

        let now_frame = cx.frame_id.0;
        let focus_visible = fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
        let pressed = enabled && st.pressed;
        let hovered = enabled && st.hovered;
        let focused = enabled && st.focused && focus_visible;
        let interaction = pressable_interaction(pressed, hovered, focused);

        cx.pressable_on_activate_for(
            pressable_id,
            Arc::new({
                let selection_model = selection_model.clone();
                move |host, action_cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&selection_model, |s| *s = selection_kind);
                    host.request_redraw(action_cx.window);
                }
            }),
        );

        // Keyboard step: adjust the time while a selector field is focused.
        cx.key_add_on_key_down_for(
            pressable_id,
            Arc::new({
                let time_on_key = time_model.clone();
                move |host, acx, down| {
                    if down.repeat {
                        return false;
                    }
                    if down.modifiers.alt || down.modifiers.ctrl || down.modifiers.meta {
                        return false;
                    }

                    let step_minutes: i32 = match selection_kind {
                        TimePickerSelection::Hour => 60,
                        TimePickerSelection::Minute => 1,
                    };

                    let delta = match down.key {
                        KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-step_minutes),
                        KeyCode::ArrowRight | KeyCode::ArrowUp => Some(step_minutes),
                        _ => None,
                    };
                    let Some(delta) = delta else { return false };

                    let cur = host
                        .models_mut()
                        .get_cloned(&time_on_key)
                        .unwrap_or_else(default_time);
                    let next = add_minutes_wrapping(cur, delta);
                    let _ = host.models_mut().update(&time_on_key, |t| *t = next);
                    host.request_redraw(acx.window);
                    true
                }
            }),
        );

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(a11y_label.clone()),
                test_id: Some(Arc::<str>::from(test_id)),
                selected,
                ..Default::default()
            },
            layout: {
                let mut l = LayoutStyle::default();
                l.overflow = Overflow::Visible;
                l.size.width = Length::Px(container_w);
                l.size.height = Length::Px(container_h);
                l
            },
            focus_ring: Some(focus_ring),
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let (
                    background,
                    label_color,
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    label_style,
                ) = {
                    let theme = Theme::global(&*cx.app);

                    let background = time_tokens::time_selector_container_color(theme, selected);
                    let label_color =
                        time_tokens::time_selector_label_color(theme, selected, interaction);

                    let (state_layer_color, state_layer_target) = match interaction {
                        Some(i @ PressableInteraction::Hovered)
                        | Some(i @ PressableInteraction::Focused) => (
                            time_tokens::time_selector_state_layer_color(theme, selected, i),
                            time_tokens::time_selector_state_layer_opacity(theme, i),
                        ),
                        _ => (Color::TRANSPARENT, 0.0),
                    };

                    let ripple_base_opacity = time_tokens::time_selector_state_layer_opacity(
                        theme,
                        PressableInteraction::Pressed,
                    );
                    let config = material_pressable_indication_config(theme, None);
                    let label_style = time_tokens::time_selector_label_text_style(theme);

                    (
                        background,
                        label_color,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        label_style,
                    )
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let mut chrome = ContainerProps::default();
                chrome.layout.overflow = Overflow::Clip;
                chrome.background = Some(background);
                chrome.corner_radii = corner_radii;
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Fill;

                let mut tp = TextProps::new(text.clone());
                tp.style = Some(label_style);
                tp.color = Some(label_color);
                tp.wrap = TextWrap::None;
                tp.overflow = TextOverflow::Clip;
                let label_el = cx.text_props(tp);

                let mut center = FlexProps::default();
                center.direction = Axis::Horizontal;
                center.justify = MainAlign::Center;
                center.align = CrossAlign::Center;
                center.wrap = false;
                center.layout.size.width = Length::Fill;
                center.layout.size.height = Length::Fill;
                let content = cx.flex(center, move |_cx| vec![label_el]);

                vec![cx.container(chrome, move |_cx| vec![overlay, content])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn time_picker_clock_dial<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    selection: TimePickerSelection,
    dial_dragging_model: Model<bool>,
    is_24h: bool,
) -> AnyElement {
    let (size, handle_size, background, corner_radii) = {
        let theme = Theme::global(&*cx.app);
        (
            time_tokens::clock_dial_size(theme),
            time_tokens::clock_dial_handle_size(theme),
            time_tokens::clock_dial_background(theme),
            time_tokens::clock_dial_shape(theme),
        )
    };

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Px(size);
    container.layout.size.height = Length::Px(size);
    container.layout.overflow = Overflow::Clip;
    container.background = Some(background);
    container.corner_radii = corner_radii;

    let (labels, selected_idx) = dial_labels(time_now, selection, is_24h);
    let center = size.0 * 0.5;
    let radius = center - (handle_size.0 * 0.5) - 8.0;

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("time-picker-clock-dial")),
            ..Default::default()
        },
        move |cx| {
            vec![cx.container(container, move |cx| {
                let mut out: Vec<AnyElement> = Vec::new();
                for (idx, (label, value)) in labels.iter().enumerate() {
                    let selected = idx == selected_idx;
                    out.push(dial_label(
                        cx,
                        *label,
                        *value,
                        idx,
                        labels.len(),
                        selected,
                        selection,
                        is_24h,
                        time_model.clone(),
                        selection_model.clone(),
                        center,
                        radius,
                        handle_size.0,
                    ));
                }

                // Pointer-driven selection (click + drag) at the dial level.
                {
                    let time_on_pointer = time_model.clone();
                    let selection_on_pointer = selection_model.clone();
                    let dragging_on_pointer = dial_dragging_model.clone();
                    let selection_for_update = selection_on_pointer.clone();

                    let update_from_pos =
                        move |host: &mut dyn UiPointerActionHost,
                              acx: ActionCx,
                              pos: fret_core::Point| {
                            let bounds = host.bounds();
                            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                                return;
                            }

                            let center = fret_core::Point::new(
                                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                            );
                            let dx = pos.x.0 - center.x.0;
                            let dy = pos.y.0 - center.y.0;
                            let r2 = dx * dx + dy * dy;
                            let radius = 0.5 * bounds.size.width.0.min(bounds.size.height.0);
                            if radius <= 0.0 || r2 <= (radius * 0.25) * (radius * 0.25) {
                                return;
                            }

                            let a = dy.atan2(dx);
                            let from_top = (a + PI * 0.5).rem_euclid(2.0 * PI);

                            let time_now = host
                                .models_mut()
                                .get_cloned(&time_on_pointer)
                                .unwrap_or_else(default_time);
                            let selection = host
                                .models_mut()
                                .get_cloned(&selection_for_update)
                                .unwrap_or_default();

                            let (labels, _) = dial_labels(time_now, selection, is_24h);
                            if labels.is_empty() {
                                return;
                            }

                            let step = 2.0 * PI / (labels.len() as f32);
                            let idx = ((from_top / step + 0.5).floor() as usize) % labels.len();
                            let value = labels[idx].1;

                            let _ = host.models_mut().update(&time_on_pointer, |t: &mut Time| {
                                *t = match selection {
                                    TimePickerSelection::Hour => {
                                        let minute = t.minute();
                                        let second = t.second();
                                        if is_24h {
                                            Time::from_hms(value.min(23) as u8, minute, second)
                                                .unwrap_or(*t)
                                        } else {
                                            let hour12 = (value % 12) as u8;
                                            let hour12 = if hour12 == 0 { 12 } else { hour12 };
                                            let hour24 = hour12_to_24(hour12, current_period(*t));
                                            Time::from_hms(hour24, minute, second).unwrap_or(*t)
                                        }
                                    }
                                    TimePickerSelection::Minute => {
                                        let hour = t.hour();
                                        let second = t.second();
                                        Time::from_hms(hour, value.min(59) as u8, second)
                                            .unwrap_or(*t)
                                    }
                                };
                            });
                            host.request_redraw(acx.window);
                        };

                    let on_down = Arc::new({
                        let update_from_pos = update_from_pos.clone();
                        let dragging_on_pointer = dragging_on_pointer.clone();
                        move |host: &mut dyn UiPointerActionHost,
                              acx: ActionCx,
                              down: PointerDownCx| {
                            if down.button != MouseButton::Left {
                                return false;
                            }
                            host.capture_pointer();
                            host.set_cursor_icon(CursorIcon::Pointer);
                            let _ = host
                                .models_mut()
                                .update(&dragging_on_pointer, |v| *v = true);
                            update_from_pos(host, acx, down.position);
                            true
                        }
                    });

                    let on_move = Arc::new({
                        let update_from_pos = update_from_pos.clone();
                        let dragging_on_pointer = dragging_on_pointer.clone();
                        move |host: &mut dyn UiPointerActionHost,
                              acx: ActionCx,
                              mv: PointerMoveCx| {
                            host.set_cursor_icon(CursorIcon::Pointer);
                            let is_dragging = host
                                .models_mut()
                                .read(&dragging_on_pointer, |v| *v)
                                .ok()
                                .unwrap_or(false);
                            if is_dragging {
                                update_from_pos(host, acx, mv.position);
                            }
                            true
                        }
                    });

                    let on_up = Arc::new({
                        let dragging_on_pointer = dragging_on_pointer.clone();
                        let selection_on_pointer = selection_on_pointer.clone();
                        move |host: &mut dyn UiPointerActionHost,
                              acx: ActionCx,
                              _up: PointerUpCx| {
                            let _ = host
                                .models_mut()
                                .update(&dragging_on_pointer, |v| *v = false);
                            host.release_pointer_capture();

                            let selection = host
                                .models_mut()
                                .get_cloned(&selection_on_pointer)
                                .unwrap_or_default();
                            if selection == TimePickerSelection::Hour {
                                let _ = host.models_mut().update(&selection_on_pointer, |s| {
                                    *s = TimePickerSelection::Minute
                                });
                            }

                            host.request_redraw(acx.window);
                            true
                        }
                    });

                    out.push(cx.pointer_region(
                        PointerRegionProps {
                            layout: {
                                let mut l = LayoutStyle::default();
                                l.position = fret_ui::element::PositionStyle::Absolute;
                                l.inset.top = Some(Px(0.0));
                                l.inset.right = Some(Px(0.0));
                                l.inset.bottom = Some(Px(0.0));
                                l.inset.left = Some(Px(0.0));
                                l.size.width = Length::Fill;
                                l.size.height = Length::Fill;
                                l
                            },
                            enabled: true,
                        },
                        move |cx| {
                            cx.pointer_region_on_pointer_down(on_down);
                            cx.pointer_region_on_pointer_move(on_move);
                            cx.pointer_region_on_pointer_up(on_up);
                            Vec::<AnyElement>::new()
                        },
                    ));
                }

                out
            })]
        },
    )
}

fn dial_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: u32,
    value: u32,
    idx: usize,
    len: usize,
    selected: bool,
    selection: TimePickerSelection,
    is_24h: bool,
    time_model: Model<Time>,
    selection_model: Model<TimePickerSelection>,
    center: f32,
    radius: f32,
    handle_size: f32,
) -> AnyElement {
    let angle = -PI * 0.5 + (idx as f32) * (2.0 * PI / (len as f32));
    let x = center + radius * angle.cos() - handle_size * 0.5;
    let y = center + radius * angle.sin() - handle_size * 0.5;

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = true;
        let pressed = enabled && st.pressed;

        cx.pressable_on_activate_for(
            pressable_id,
            Arc::new({
                let time_model = time_model.clone();
                let selection_model = selection_model.clone();
                move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&time_model, |t: &mut Time| {
                        *t = match selection {
                            TimePickerSelection::Hour => {
                                let minute = t.minute();
                                let second = t.second();
                                if is_24h {
                                    Time::from_hms(value as u8, minute, second).unwrap_or(*t)
                                } else {
                                    let cur = t.hour();
                                    let is_pm = cur >= 12;
                                    let base = (value % 12) as u8;
                                    let hour24 = if is_pm { (base % 12) + 12 } else { base % 12 };
                                    Time::from_hms(hour24, minute, second).unwrap_or(*t)
                                }
                            }
                            TimePickerSelection::Minute => {
                                let hour = t.hour();
                                let second = t.second();
                                Time::from_hms(hour, (value.min(59)) as u8, second).unwrap_or(*t)
                            }
                        };
                    });

                    if selection == TimePickerSelection::Hour {
                        let _ = host
                            .models_mut()
                            .update(&selection_model, |s| *s = TimePickerSelection::Minute);
                    }

                    host.request_redraw(action_cx.window);
                }
            }),
        );

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::<str>::from(format!("{label}"))),
                selected,
                ..Default::default()
            },
            layout: {
                let mut l = LayoutStyle::default();
                l.position = fret_ui::element::PositionStyle::Absolute;
                l.inset.left = Some(Px(x));
                l.inset.top = Some(Px(y));
                l.size.width = Length::Px(Px(handle_size));
                l.size.height = Length::Px(Px(handle_size));
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: None,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let (handle_color, handle_shape, label_color, label_style) = {
                    let theme = Theme::global(&*cx.app);
                    let handle_color = time_tokens::clock_dial_handle_color(theme);
                    let handle_shape = time_tokens::clock_dial_handle_shape(theme);
                    let label_color = time_tokens::clock_dial_label_text_color(theme, selected);
                    let label_style = time_tokens::clock_dial_label_text_style(theme);
                    (handle_color, handle_shape, label_color, label_style)
                };

                let mut container = ContainerProps::default();
                container.layout.size.width = Length::Fill;
                container.layout.size.height = Length::Fill;
                container.layout.overflow = Overflow::Clip;
                container.background = selected.then_some(handle_color);
                container.corner_radii = handle_shape;

                let mut label_text = if selection == TimePickerSelection::Minute {
                    format!("{label:02}")
                } else {
                    format!("{label}")
                };
                if label_text == "00" {
                    label_text = "0".to_string();
                }
                let mut tp = TextProps::new(Arc::<str>::from(label_text));
                tp.style = Some(label_style);
                tp.color = Some(label_color);
                tp.wrap = TextWrap::None;
                tp.overflow = TextOverflow::Clip;
                let label_el = cx.text_props(tp);

                let mut center_flex = FlexProps::default();
                center_flex.direction = Axis::Horizontal;
                center_flex.justify = MainAlign::Center;
                center_flex.align = CrossAlign::Center;
                center_flex.wrap = false;
                center_flex.layout.size.width = Length::Fill;
                center_flex.layout.size.height = Length::Fill;
                let content = cx.flex(center_flex, move |_cx| vec![label_el]);

                // No ripple: Material Web/Compose dial is gesture-driven; we keep this MVP click-only.
                // We still surface "pressed" via background selection, and keep the hitbox stable.
                let _ = pressed;

                vec![cx.container(container, move |_cx| vec![content])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Period {
    Am,
    Pm,
}

fn current_period(time_now: Time) -> Period {
    if time_now.hour() >= 12 {
        Period::Pm
    } else {
        Period::Am
    }
}

fn time_input_period_selector<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
) -> AnyElement {
    let (width, height, outline_width, outline_color, corner_radii) = {
        let theme = Theme::global(&*cx.app);
        (
            time_input_tokens::period_selector_container_width(theme),
            time_input_tokens::period_selector_container_height(theme),
            time_input_tokens::period_selector_outline_width(theme),
            time_input_tokens::period_selector_outline_color(theme),
            time_input_tokens::period_selector_shape(theme),
        )
    };

    let current = current_period(time_now);

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Px(width);
    container.layout.size.height = Length::Px(height);
    container.layout.overflow = Overflow::Clip;
    container.border = Edges::all(outline_width);
    container.border_color = Some(outline_color);
    container.corner_radii = corner_radii;

    let mut flex = FlexProps::default();
    flex.direction = Axis::Vertical;
    flex.justify = MainAlign::Start;
    flex.align = CrossAlign::Stretch;
    flex.wrap = false;
    flex.layout.size.width = Length::Fill;
    flex.layout.size.height = Length::Fill;

    cx.container(container, move |cx| {
        let am = time_input_period_item(
            cx,
            "AM",
            current == Period::Am,
            time_model.clone(),
            Period::Am,
        );
        let pm = time_input_period_item(
            cx,
            "PM",
            current == Period::Pm,
            time_model.clone(),
            Period::Pm,
        );
        vec![cx.flex(flex, move |_cx| vec![am, pm])]
    })
}

fn time_input_period_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    selected: bool,
    time_model: Model<Time>,
    period: Period,
) -> AnyElement {
    let corner_radii = Corners::all(Px(0.0));
    let focus_ring = {
        let theme = Theme::global(&*cx.app);
        material_focus_ring_for_component(theme, time_input_tokens::COMPONENT_PREFIX, corner_radii)
    };

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = true;

        let now_frame = cx.frame_id.0;
        let focus_visible = fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
        let pressed = enabled && st.pressed;
        let hovered = enabled && st.hovered;
        let focused = enabled && st.focused && focus_visible;
        let interaction = pressable_interaction(pressed, hovered, focused);

        cx.pressable_on_activate_for(
            pressable_id,
            Arc::new({
                let time_model = time_model.clone();
                move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&time_model, |t: &mut Time| {
                        let hour = t.hour();
                        let minute = t.minute();
                        let second = t.second();
                        let new_hour = match period {
                            Period::Am => {
                                if hour >= 12 {
                                    hour - 12
                                } else {
                                    hour
                                }
                            }
                            Period::Pm => {
                                if hour < 12 {
                                    hour + 12
                                } else {
                                    hour
                                }
                            }
                        };
                        *t = Time::from_hms(new_hour, minute, second).unwrap_or(*t);
                    });
                    host.request_redraw(action_cx.window);
                }
            }),
        );

        // Split the 72px container into two equal rows.
        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::<str>::from(label)),
                selected,
                ..Default::default()
            },
            layout: {
                let mut l = LayoutStyle::default();
                l.overflow = Overflow::Visible;
                l.size.width = Length::Fill;
                l.size.height = Length::Fill;
                l
            },
            focus_ring: Some(focus_ring),
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let (
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    background,
                    label_color,
                    label_style,
                ) = {
                    let theme = Theme::global(&*cx.app);

                    let (state_layer_color, state_layer_target) = match interaction {
                        Some(i @ PressableInteraction::Hovered)
                        | Some(i @ PressableInteraction::Focused) => (
                            time_input_tokens::period_selector_state_layer_color(
                                theme, selected, i,
                            ),
                            time_input_tokens::period_selector_state_layer_opacity(theme, i),
                        ),
                        _ => (Color::TRANSPARENT, 0.0),
                    };

                    let ripple_base_opacity =
                        time_input_tokens::period_selector_state_layer_opacity(
                            theme,
                            PressableInteraction::Pressed,
                        );
                    let config = material_pressable_indication_config(theme, None);

                    let background = selected.then_some(
                        time_input_tokens::period_selector_selected_container_color(theme),
                    );
                    let label_color = time_input_tokens::period_selector_label_color(
                        theme,
                        selected,
                        interaction,
                    );
                    let label_style = time_input_tokens::period_selector_label_text_style(theme);

                    (
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        background,
                        label_color,
                        label_style,
                    )
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let mut chrome = ContainerProps::default();
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Fill;
                chrome.background = background;

                let mut tp = TextProps::new(Arc::<str>::from(label));
                tp.style = Some(label_style);
                tp.color = Some(label_color);
                tp.wrap = TextWrap::None;
                tp.overflow = TextOverflow::Clip;
                let label_el = cx.text_props(tp);

                let mut center = FlexProps::default();
                center.direction = Axis::Horizontal;
                center.justify = MainAlign::Center;
                center.align = CrossAlign::Center;
                center.wrap = false;
                center.layout.size.width = Length::Fill;
                center.layout.size.height = Length::Fill;
                let content = cx.flex(center, move |_cx| vec![label_el]);

                vec![cx.container(chrome, move |_cx| vec![overlay, content])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn time_picker_period_selector<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
) -> AnyElement {
    let (width, height, outline_width, outline_color, corner_radii) = {
        let theme = Theme::global(&*cx.app);
        (
            time_tokens::period_selector_container_width(theme),
            time_tokens::period_selector_container_height(theme),
            time_tokens::period_selector_outline_width(theme),
            time_tokens::period_selector_outline_color(theme),
            time_tokens::period_selector_shape(theme),
        )
    };

    let current = current_period(time_now);

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Px(width);
    container.layout.size.height = Length::Px(height);
    container.layout.overflow = Overflow::Clip;
    container.border = Edges::all(outline_width);
    container.border_color = Some(outline_color);
    container.corner_radii = corner_radii;

    let mut flex = FlexProps::default();
    flex.direction = Axis::Vertical;
    flex.justify = MainAlign::Start;
    flex.align = CrossAlign::Stretch;
    flex.wrap = false;
    flex.layout.size.width = Length::Fill;
    flex.layout.size.height = Length::Fill;

    cx.container(container, move |cx| {
        let am = period_item(
            cx,
            "AM",
            current == Period::Am,
            time_model.clone(),
            Period::Am,
        );
        let pm = period_item(
            cx,
            "PM",
            current == Period::Pm,
            time_model.clone(),
            Period::Pm,
        );
        vec![cx.flex(flex, move |_cx| vec![am, pm])]
    })
}

fn period_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    selected: bool,
    time_model: Model<Time>,
    period: Period,
) -> AnyElement {
    let corner_radii = Corners::all(Px(0.0));
    let focus_ring = {
        let theme = Theme::global(&*cx.app);
        material_focus_ring_for_component(theme, time_tokens::COMPONENT_PREFIX, corner_radii)
    };

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = true;

        let now_frame = cx.frame_id.0;
        let focus_visible = fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
        let pressed = enabled && st.pressed;
        let hovered = enabled && st.hovered;
        let focused = enabled && st.focused && focus_visible;
        let interaction = pressable_interaction(pressed, hovered, focused);

        cx.pressable_on_activate_for(
            pressable_id,
            Arc::new({
                let time_model = time_model.clone();
                move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&time_model, |t: &mut Time| {
                        let hour = t.hour();
                        let minute = t.minute();
                        let second = t.second();
                        let new_hour = match period {
                            Period::Am => {
                                if hour >= 12 {
                                    hour - 12
                                } else {
                                    hour
                                }
                            }
                            Period::Pm => {
                                if hour < 12 {
                                    hour + 12
                                } else {
                                    hour
                                }
                            }
                        };
                        *t = Time::from_hms(new_hour, minute, second).unwrap_or(*t);
                    });
                    host.request_redraw(action_cx.window);
                }
            }),
        );

        // Split the 80px container into two equal rows.
        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::<str>::from(label)),
                selected,
                ..Default::default()
            },
            layout: {
                let mut l = LayoutStyle::default();
                l.overflow = Overflow::Visible;
                l.size.width = Length::Fill;
                l.size.height = Length::Fill;
                l
            },
            focus_ring: Some(focus_ring),
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let (
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    background,
                    label_color,
                    label_style,
                ) = {
                    let theme = Theme::global(&*cx.app);

                    let (state_layer_color, state_layer_target) = match interaction {
                        Some(i @ PressableInteraction::Hovered)
                        | Some(i @ PressableInteraction::Focused) => (
                            time_tokens::period_selector_state_layer_color(theme, selected, i),
                            time_tokens::period_selector_state_layer_opacity(theme, i),
                        ),
                        _ => (Color::TRANSPARENT, 0.0),
                    };

                    let ripple_base_opacity = time_tokens::period_selector_state_layer_opacity(
                        theme,
                        PressableInteraction::Pressed,
                    );
                    let config = material_pressable_indication_config(theme, None);

                    let background = selected
                        .then_some(time_tokens::period_selector_selected_container_color(theme));
                    let label_color =
                        time_tokens::period_selector_label_color(theme, selected, interaction);
                    let label_style = time_tokens::period_selector_label_text_style(theme);

                    (
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        background,
                        label_color,
                        label_style,
                    )
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let mut chrome = ContainerProps::default();
                chrome.layout.overflow = Overflow::Clip;
                chrome.background = background;
                chrome.corner_radii = corner_radii;
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Fill;

                let mut tp = TextProps::new(Arc::<str>::from(label));
                tp.style = Some(label_style);
                tp.color = Some(label_color);
                tp.wrap = TextWrap::None;
                tp.overflow = TextOverflow::Clip;
                let label_el = cx.text_props(tp);

                let mut center = FlexProps::default();
                center.direction = Axis::Horizontal;
                center.justify = MainAlign::Center;
                center.align = CrossAlign::Center;
                center.wrap = false;
                center.layout.size.width = Length::Fill;
                center.layout.size.height = Length::Fill;
                let content = cx.flex(center, move |_cx| vec![label_el]);

                vec![cx.container(chrome, move |_cx| vec![overlay, content])]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn time_picker_actions<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    on_cancel: OnActivate,
    on_confirm: OnActivate,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.justify = MainAlign::End;
    props.align = CrossAlign::Center;
    props.wrap = false;
    props.gap = Px(8.0);

    cx.flex(props, move |cx| {
        let cancel = Button::new("Cancel")
            .variant(ButtonVariant::Text)
            .on_activate(on_cancel.clone())
            .test_id("time-picker-cancel")
            .into_element(cx);
        let ok = Button::new("OK")
            .variant(ButtonVariant::Text)
            .on_activate(on_confirm.clone())
            .test_id("time-picker-ok")
            .into_element(cx);
        vec![cancel, ok]
    })
}

fn ensure_selection_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_rt: fn() -> DockedRuntime,
) -> Model<TimePickerSelection> {
    let existing = cx.with_state(default_rt, |st| st.selection.clone());
    match existing {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(TimePickerSelection::Hour);
            cx.with_state(default_rt, |st| st.selection = Some(m.clone()));
            m
        }
    }
}

fn ensure_display_mode_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_rt: fn() -> DockedRuntime,
    initial: TimePickerDisplayMode,
) -> Model<TimePickerDisplayMode> {
    let existing = cx.with_state(default_rt, |st| st.display_mode.clone());
    match existing {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(initial);
            cx.with_state(default_rt, |st| st.display_mode = Some(m.clone()));
            m
        }
    }
}

fn ensure_dial_dragging_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_rt: fn() -> DockedRuntime,
) -> Model<bool> {
    let existing = cx.with_state(default_rt, |st| st.dial_dragging.clone());
    match existing {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(false);
            cx.with_state(default_rt, |st| st.dial_dragging = Some(m.clone()));
            m
        }
    }
}

fn ensure_time_input_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_rt: fn() -> DockedRuntime,
    time_now: Time,
    is_24h: bool,
) -> (Model<String>, Model<String>) {
    let existing = cx.with_state(default_rt, |st| {
        st.input_hour.clone().zip(st.input_minute.clone())
    });
    match existing {
        Some(models) => models,
        None => {
            let (hour, minute) = time_to_display(time_now, is_24h);
            let hour_m = cx.app.models_mut().insert(format!("{hour:02}"));
            let minute_m = cx.app.models_mut().insert(format!("{minute:02}"));
            cx.with_state(default_rt, |st| {
                st.input_hour = Some(hour_m.clone());
                st.input_minute = Some(minute_m.clone());
            });
            (hour_m, minute_m)
        }
    }
}

fn ensure_time_input_edit_state_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_rt: fn() -> DockedRuntime,
) -> Model<TimeInputEditState> {
    let existing = cx.with_state(default_rt, |st| st.time_input_edit.clone());
    match existing {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(TimeInputEditState::default());
            cx.with_state(default_rt, |st| st.time_input_edit = Some(m.clone()));
            m
        }
    }
}

fn with_alpha(mut c: Color, alpha: f32) -> Color {
    c.a = alpha;
    c
}

fn apply_time_input_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    time_now: Time,
    time_model: Model<Time>,
    input_hour: Model<String>,
    input_minute: Model<String>,
    is_24h: bool,
) {
    let hour_raw = cx
        .get_model_cloned(&input_hour, Invalidation::Layout)
        .unwrap_or_default();
    let minute_raw = cx
        .get_model_cloned(&input_minute, Invalidation::Layout)
        .unwrap_or_default();

    let hour_s = sanitize_time_input_digits(&hour_raw);
    if hour_s != hour_raw {
        let _ = cx
            .app
            .models_mut()
            .update(&input_hour, |v| *v = hour_s.clone());
    }
    let minute_s = sanitize_time_input_digits(&minute_raw);
    if minute_s != minute_raw {
        let _ = cx
            .app
            .models_mut()
            .update(&input_minute, |v| *v = minute_s.clone());
    }

    let hour_val = parse_u8(&hour_s);
    let minute_val = parse_u8(&minute_s);

    if hour_val.is_none() && minute_val.is_none() {
        return;
    }

    let next_minute = minute_val.map(|m| m.min(59)).unwrap_or(time_now.minute());

    let next_hour = if is_24h {
        hour_val.map(|h| h.min(23)).unwrap_or(time_now.hour())
    } else {
        let period = current_period(time_now);
        let hour12 = hour_val.map(|h| if h == 0 { 12 } else { h.clamp(1, 12) });
        match hour12 {
            Some(hour12) => hour12_to_24(hour12, period),
            None => time_now.hour(),
        }
    };

    let next = Time::from_hms(next_hour, next_minute, time_now.second()).unwrap_or(time_now);
    if next != time_now {
        let _ = cx.app.models_mut().update(&time_model, |t| *t = next);
    }
}

fn sanitize_time_input_digits(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
            if out.len() >= 2 {
                break;
            }
        }
    }
    out
}

fn parse_u8(raw: &str) -> Option<u8> {
    if raw.is_empty() {
        return None;
    }
    raw.parse::<u8>().ok()
}

fn hour12_to_24(hour12: u8, period: Period) -> u8 {
    let hour = hour12 % 12;
    match period {
        Period::Am => hour,
        Period::Pm => hour + 12,
    }
}

fn add_minutes_wrapping(time: Time, delta_minutes: i32) -> Time {
    let total = (time.hour() as i32) * 60 + (time.minute() as i32);
    let next = (total + delta_minutes).rem_euclid(24 * 60);
    let hour = (next / 60) as u8;
    let minute = (next % 60) as u8;
    Time::from_hms(hour, minute, time.second()).unwrap_or(time)
}

fn default_time() -> Time {
    Time::from_hms(9, 41, 0).expect("valid time")
}

fn time_to_display(time_now: Time, is_24h: bool) -> (u8, u8) {
    if is_24h {
        return (time_now.hour(), time_now.minute());
    }
    let h = time_now.hour();
    let hour12 = match h % 12 {
        0 => 12,
        v => v,
    };
    (hour12, time_now.minute())
}

fn dial_labels(
    time_now: Time,
    selection: TimePickerSelection,
    is_24h: bool,
) -> (Vec<(u32, u32)>, usize) {
    match selection {
        TimePickerSelection::Hour => {
            if is_24h {
                let hour = time_now.hour() as usize;
                let labels: Vec<(u32, u32)> = (0..24).map(|h| (h as u32, h as u32)).collect();
                (labels, hour.min(23))
            } else {
                let hour = time_now.hour() % 12;
                let hour = if hour == 0 { 12 } else { hour };
                let labels: Vec<(u32, u32)> = [12u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
                    .into_iter()
                    .map(|h| (h, h))
                    .collect();
                let idx = labels
                    .iter()
                    .position(|(l, _)| *l == hour as u32)
                    .unwrap_or(0);
                (labels, idx)
            }
        }
        TimePickerSelection::Minute => {
            let minute = time_now.minute();
            let snapped = ((minute as u32 + 2) / 5) * 5;
            let snapped = snapped.min(55);
            let labels: Vec<(u32, u32)> = (0..12).map(|i| (i * 5, i * 5)).collect();
            let idx = labels.iter().position(|(l, _)| *l == snapped).unwrap_or(0);
            (labels, idx)
        }
    }
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
