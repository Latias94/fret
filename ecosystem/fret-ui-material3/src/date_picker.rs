//! Material 3 date picker primitives (P2).
//!
//! Outcome-oriented implementation:
//! - Token-driven container + day cell outcomes via `md.comp.date-picker.{docked,modal}.*`.
//! - Modal variant uses `OverlayRequest::modal` with a scrim and focus trap/restore.
//! - Selection is staged while open and applied on confirm.

use std::sync::Arc;
use std::sync::OnceLock;

use fret_core::{Axis, Color, Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{DismissReason, DismissRequestCx, OnActivate, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::headless::calendar::{CalendarMonth, month_grid};
use fret_ui_kit::overlay_controller;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::{OverlayController, OverlayPresence};
use time::{Date, OffsetDateTime, Weekday};

use crate::button::{Button, ButtonVariant};
use crate::foundation::surface::material_surface_style;
use crate::motion;
use crate::tokens::date_picker as date_tokens;
use crate::tokens::date_picker::DatePickerTokenVariant;

fn default_date_picker_test_id() -> Arc<str> {
    static ID: OnceLock<Arc<str>> = OnceLock::new();
    ID.get_or_init(|| Arc::<str>::from("material3-date-picker"))
        .clone()
}

fn cached_day_of_month_label(day: u8) -> Arc<str> {
    static TABLE: OnceLock<Vec<Arc<str>>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        (1u8..=31)
            .map(|d| Arc::<str>::from(d.to_string()))
            .collect::<Vec<_>>()
    });
    let idx = day.saturating_sub(1) as usize;
    table
        .get(idx)
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from(day.to_string()))
}

fn weekday_short_arc(w: Weekday) -> Arc<str> {
    static TABLE: OnceLock<Vec<Arc<str>>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
            .into_iter()
            .map(Arc::<str>::from)
            .collect::<Vec<_>>()
    });
    let idx = match w {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };
    table[idx].clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatePickerVariant {
    #[default]
    Docked,
    Modal,
}

#[derive(Clone)]
pub struct DockedDatePicker {
    variant: DatePickerVariant,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
    week_start: Weekday,
    today: Option<Date>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DockedDatePicker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockedDatePicker")
            .field("variant", &self.variant)
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("week_start", &self.week_start)
            .field("today", &self.today)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl DockedDatePicker {
    pub fn new(month: Model<CalendarMonth>, selected: Model<Option<Date>>) -> Self {
        Self {
            variant: DatePickerVariant::Docked,
            month,
            selected,
            week_start: Weekday::Monday,
            today: None,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: DatePickerVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = week_start;
        self
    }

    pub fn today(mut self, today: Option<Date>) -> Self {
        self.today = today;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let month = cx
                .get_model_cloned(&self.month, Invalidation::Layout)
                .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
            let selected_now = cx
                .get_model_cloned(&self.selected, Invalidation::Layout)
                .unwrap_or(None);

            let today = self
                .today
                .unwrap_or_else(|| OffsetDateTime::now_utc().date());
            let token_variant = match self.variant {
                DatePickerVariant::Docked => DatePickerTokenVariant::Docked,
                DatePickerVariant::Modal => DatePickerTokenVariant::Modal,
            };

            let (width, height, background, shadow, corner_radii) = {
                let theme = Theme::global(&*cx.app);
                let width = date_tokens::container_width(theme, token_variant);
                let height = date_tokens::container_height(theme, token_variant);
                let container_color = date_tokens::container_color(theme, token_variant);
                let elevation = date_tokens::container_elevation(theme, token_variant);
                let corner_radii = date_tokens::container_shape(theme, token_variant);
                let surface =
                    material_surface_style(theme, container_color, elevation, None, corner_radii);
                (
                    width,
                    height,
                    surface.background,
                    surface.shadow,
                    corner_radii,
                )
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(width);
            layout.size.height = Length::Px(height);
            layout.overflow = Overflow::Clip;

            let mut container = ContainerProps::default();
            container.layout = layout;
            container.background = Some(background);
            container.shadow = shadow;
            container.corner_radii = corner_radii;

            let content = date_picker_body(
                cx,
                token_variant,
                month,
                self.month.clone(),
                self.week_start,
                today,
                selected_now,
                self.selected.clone(),
                self.test_id.clone(),
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

#[derive(Clone)]
pub struct DatePickerDialog {
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
    today: Option<Date>,
    scrim_opacity: f32,
    open_duration_ms: Option<u32>,
    close_duration_ms: Option<u32>,
    easing_key: Option<Arc<str>>,
    on_dismiss_request: Option<OnDismissRequest>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for DatePickerDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatePickerDialog")
            .field("open", &"<model>")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("today", &self.today)
            .field("scrim_opacity", &self.scrim_opacity)
            .field("open_duration_ms", &self.open_duration_ms)
            .field("close_duration_ms", &self.close_duration_ms)
            .field("easing_key", &self.easing_key)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

#[derive(Default)]
struct DialogRuntime {
    models: Option<DialogModels>,
    was_open: bool,
}

#[derive(Clone)]
struct DialogModels {
    draft_month: Model<CalendarMonth>,
    draft_selected: Model<Option<Date>>,
}

impl DatePickerDialog {
    pub fn new(
        open: Model<bool>,
        month: Model<CalendarMonth>,
        selected: Model<Option<Date>>,
    ) -> Self {
        Self {
            open,
            month,
            selected,
            today: None,
            // Align with Dialog defaults.
            scrim_opacity: 0.32,
            open_duration_ms: None,
            close_duration_ms: None,
            easing_key: Some(Arc::<str>::from("md.sys.motion.easing.emphasized")),
            on_dismiss_request: None,
            test_id: None,
        }
    }

    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn today(mut self, today: Option<Date>) -> Self {
        self.today = today;
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

    #[track_caller]
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
                    let today = OffsetDateTime::now_utc().date();
                    let draft_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
                    let draft_selected = cx.app.models_mut().insert(None::<Date>);
                    let models = DialogModels {
                        draft_month,
                        draft_selected,
                    };
                    cx.with_state(DialogRuntime::default, |st| st.models = Some(models.clone()));
                    models
                }
            };

            if open_now && !prev_open {
                let external_month = cx
                    .get_model_cloned(&self.month, Invalidation::Layout)
                    .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
                let external_selected = cx
                    .get_model_cloned(&self.selected, Invalidation::Layout)
                    .unwrap_or(None);

                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.draft_month, |m| *m = external_month);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&models.draft_selected, |m| *m = external_selected);
            }

            let easing_key = self
                .easing_key
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("md.sys.motion.easing.emphasized"));

            let (theme_motion_ms, bezier) = {
                let theme = Theme::global(&*cx.app);
                let motion_ms = theme.duration_ms_by_key("md.sys.motion.duration.medium2");
                let bezier =
                    theme
                        .easing_by_key(easing_key.as_ref())
                        .unwrap_or(fret_ui::theme::CubicBezier {
                            x1: 0.0,
                            y1: 0.0,
                            x2: 1.0,
                            y2: 1.0,
                        });
                (motion_ms, bezier)
            };

            let open_ms = self
                .open_duration_ms
                .or(theme_motion_ms)
                .unwrap_or(300);
            let close_ms = self
                .close_duration_ms
                .or(theme_motion_ms)
                .unwrap_or(300);
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
                let scrim_base = {
                    let theme = Theme::global(&*cx.app);
                    theme.color_required("md.sys.color.scrim")
                };
                let scrim_alpha = (scrim_base.a * self.scrim_opacity * transition.progress)
                    .clamp(0.0, 1.0);
                let scrim_color = with_alpha(scrim_base, scrim_alpha);

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

                let cancel: OnActivate = {
                    let open = self.open.clone();
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.request_redraw(action_cx.window);
                    })
                };
                let confirm: OnActivate = {
                    let open = self.open.clone();
                    let selected = self.selected.clone();
                    let month = self.month.clone();
                    let draft_selected = models.draft_selected.clone();
                    let draft_month = models.draft_month.clone();

                    Arc::new(move |host, action_cx, _reason| {
                        let draft_selected_value = host
                            .models_mut()
                            .read(&draft_selected, |v| *v)
                            .ok()
                            .flatten();
                        let draft_month_value =
                            host.models_mut().read(&draft_month, |v| *v).ok();

                        if let Some(v) = draft_selected_value {
                            let _ = host.models_mut().update(&selected, |s| *s = Some(v));
                        }
                        if let Some(m) = draft_month_value {
                            let _ = host.models_mut().update(&month, |mm| *mm = m);
                        }
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.request_redraw(action_cx.window);
                    })
                };

                let today = self.today.unwrap_or_else(|| OffsetDateTime::now_utc().date());

                let overlay_root = cx.named("material3_date_picker_dialog_root", |cx| {
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
                                let scale = 0.95 + 0.05 * transition.progress;
                                let transform = fret_core::Transform2D::scale_uniform(scale);

                                let mut align = FlexProps::default();
                                align.direction = Axis::Vertical;
                                align.justify = MainAlign::Center;
                                align.align = CrossAlign::Center;
                                align.wrap = false;
                                align.layout.size.width = Length::Fill;
                                align.layout.size.height = Length::Fill;

                                let picker = date_picker_modal_panel(
                                    cx,
                                    models.draft_month.clone(),
                                    models.draft_selected.clone(),
                                    panel_test_id.clone(),
                                    today,
                                    cancel.clone(),
                                    confirm.clone(),
                                );
                                let trapped = focus_scope_prim::focus_trap(cx, move |_cx| {
                                    vec![picker]
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
                    self.open.clone(),
                    presence,
                    vec![overlay_root],
                );
                request.root_name =
                    Some(format!("material3.date_picker_dialog.{}", overlay_id.0));
                request.close_on_window_focus_lost = true;
                request.close_on_window_resize = true;
                request.dismissible_on_dismiss_request = Some(dismiss_handler_for_request);
                OverlayController::request(cx, request);
            }

            underlay_el
        })
    }
}

fn date_picker_modal_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
    test_id: Option<Arc<str>>,
    today: Date,
    on_cancel: OnActivate,
    on_confirm: OnActivate,
) -> AnyElement {
    let token_variant = DatePickerTokenVariant::Modal;
    let month_value = cx
        .get_model_cloned(&month, Invalidation::Layout)
        .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
    let selected_now = cx
        .get_model_cloned(&selected, Invalidation::Layout)
        .unwrap_or(None);

    let (width, height, background, shadow, corner_radii, headline_style, headline_color) = {
        let theme = Theme::global(&*cx.app);
        let width = date_tokens::container_width(theme, token_variant);
        let height = date_tokens::container_height(theme, token_variant);
        let container_color = date_tokens::container_color(theme, token_variant);
        let elevation = date_tokens::container_elevation(theme, token_variant);
        let corner_radii = date_tokens::container_shape(theme, token_variant);
        let surface = material_surface_style(theme, container_color, elevation, None, corner_radii);
        let headline_style = date_tokens::header_headline_style(theme);
        let headline_color = date_tokens::header_headline_color(theme);
        (
            width,
            height,
            surface.background,
            surface.shadow,
            corner_radii,
            headline_style,
            headline_color,
        )
    };

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(width);
    layout.size.height = Length::Px(height);
    layout.overflow = Overflow::Clip;

    let mut container = ContainerProps::default();
    container.layout = layout;
    container.background = Some(background);
    container.shadow = shadow;
    container.corner_radii = corner_radii;

    let title = {
        let mut props = TextProps::new(Arc::<str>::from("Select date"));
        props.style = Some(headline_style);
        props.color = Some(headline_color);
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Ellipsis;
        cx.text_props(props)
    };

    let test_id_for_body = test_id.clone();
    let body = cx.flex(
        FlexProps {
            direction: Axis::Vertical,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
            gap: Px(12.0),
            layout: {
                let mut l = LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Fill;
                l
            },
            padding: Edges::all(Px(16.0)),
        },
        move |cx| {
            vec![
                title,
                date_picker_body(
                    cx,
                    token_variant,
                    month_value,
                    month.clone(),
                    Weekday::Monday,
                    today,
                    selected_now,
                    selected.clone(),
                    test_id_for_body.clone(),
                ),
                date_picker_actions(cx, on_cancel.clone(), on_confirm.clone()),
            ]
        },
    );

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Dialog,
            test_id,
            ..Default::default()
        },
        move |cx| vec![cx.container(container, move |_cx| vec![body])],
    )
}

fn date_picker_actions<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    on_cancel: OnActivate,
    on_confirm: OnActivate,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.justify = MainAlign::End;
    props.align = CrossAlign::Center;
    props.wrap = false;
    props.gap = Px(12.0);
    props.layout.size.width = Length::Fill;

    cx.flex(props, move |cx| {
        vec![
            Button::new("Cancel")
                .variant(ButtonVariant::Text)
                .on_activate(on_cancel.clone())
                .test_id("material3-date-picker-cancel")
                .into_element(cx),
            Button::new("OK")
                .variant(ButtonVariant::Filled)
                .on_activate(on_confirm.clone())
                .test_id("material3-date-picker-confirm")
                .into_element(cx),
        ]
    })
}

fn date_picker_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    token_variant: DatePickerTokenVariant,
    month: CalendarMonth,
    month_model: Model<CalendarMonth>,
    week_start: Weekday,
    today: Date,
    selected_now: Option<Date>,
    selected_model: Model<Option<Date>>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    cx.flex(
        FlexProps {
            direction: Axis::Vertical,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
            gap: Px(8.0),
            layout: {
                let mut l = LayoutStyle::default();
                l.size.width = Length::Fill;
                l
            },
            ..Default::default()
        },
        move |cx| {
            vec![
                month_nav_header(
                    cx,
                    token_variant,
                    month,
                    month_model.clone(),
                    test_id.clone(),
                ),
                weekdays_row(cx, token_variant, week_start),
                dates_grid(
                    cx,
                    token_variant,
                    month,
                    month_model,
                    week_start,
                    today,
                    selected_now,
                    selected_model,
                    test_id,
                ),
            ]
        },
    )
}

fn month_nav_header<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    token_variant: DatePickerTokenVariant,
    month: CalendarMonth,
    month_model: Model<CalendarMonth>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    #[derive(Default)]
    struct DerivedTitle {
        month: Option<time::Month>,
        year: i32,
        title: Option<Arc<str>>,
    }

    let title = cx.with_state(DerivedTitle::default, |st| {
        if st.title.is_none() || st.month != Some(month.month) || st.year != month.year {
            st.month = Some(month.month);
            st.year = month.year;
            st.title = Some(Arc::<str>::from(format!(
                "{} {}",
                month_name_en(month.month),
                month.year
            )));
        }
        st.title.as_ref().expect("title").clone()
    });

    let mut row = FlexProps::default();
    row.direction = Axis::Horizontal;
    row.justify = MainAlign::SpaceBetween;
    row.align = CrossAlign::Center;
    row.wrap = false;
    row.layout.size.width = Length::Fill;
    row.gap = Px(12.0);

    let title_el = {
        let (style, color) = {
            let theme = Theme::global(&*cx.app);
            let style = theme
                .text_style_by_key("md.sys.typescale.title-large")
                .or_else(|| theme.text_style_by_key("md.sys.typescale.title-medium"));
            let color = theme.color_required("md.sys.color.on-surface");
            (style, color)
        };

        let mut props = TextProps::new(title);
        props.style = style;
        props.color = Some(color);
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Ellipsis;
        cx.text_props(props)
    };

    let base_id = test_id.clone().unwrap_or_else(default_date_picker_test_id);

    let prev: OnActivate = {
        let month_model = month_model.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&month_model, |m| *m = m.prev_month());
            host.request_redraw(action_cx.window);
        })
    };
    let next: OnActivate = {
        let month_model = month_model.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&month_model, |m| *m = m.next_month());
            host.request_redraw(action_cx.window);
        })
    };

    let tag = match token_variant {
        DatePickerTokenVariant::Docked => "docked",
        DatePickerTokenVariant::Modal => "modal",
    };

    #[derive(Default)]
    struct DerivedNavTestIds {
        base: Option<Arc<str>>,
        tag: Option<&'static str>,
        prev: Option<Arc<str>>,
        next: Option<Arc<str>>,
    }

    let (prev_test_id, next_test_id) = cx.with_state(DerivedNavTestIds::default, |st| {
        if st.prev.is_none() || st.base.as_deref() != Some(base_id.as_ref()) || st.tag != Some(tag)
        {
            st.base = Some(base_id.clone());
            st.tag = Some(tag);
            st.prev = Some(Arc::from(format!("{base_id}-{tag}-prev")));
            st.next = Some(Arc::from(format!("{base_id}-{tag}-next")));
        }
        (
            st.prev.as_ref().expect("prev").clone(),
            st.next.as_ref().expect("next").clone(),
        )
    });

    let prev = Button::new("Prev")
        .variant(ButtonVariant::Text)
        .on_activate(prev)
        .test_id(prev_test_id)
        .into_element(cx);
    let next = Button::new("Next")
        .variant(ButtonVariant::Text)
        .on_activate(next)
        .test_id(next_test_id)
        .into_element(cx);

    cx.flex(row, move |_cx| vec![prev, title_el, next])
}

fn weekdays_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    token_variant: DatePickerTokenVariant,
    week_start: Weekday,
) -> AnyElement {
    let mut row = FlexProps::default();
    row.direction = Axis::Horizontal;
    row.justify = MainAlign::SpaceBetween;
    row.align = CrossAlign::Center;
    row.wrap = false;
    row.layout.size.width = Length::Fill;
    row.gap = Px(0.0);

    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = date_tokens::weekdays_label_text_style(theme, token_variant);
        let color = date_tokens::weekdays_label_text_color(theme, token_variant);
        (style, color)
    };

    let weekdays = weekdays_from_start(week_start);
    cx.flex(row, move |cx| {
        weekdays
            .into_iter()
            .map(|w| {
                let label = weekday_short_arc(w);
                let mut props = TextProps::new(label);
                props.style = Some(style.clone());
                props.color = Some(color);
                props.wrap = TextWrap::None;
                props.overflow = TextOverflow::Clip;
                cx.text_props(props)
            })
            .collect::<Vec<_>>()
    })
}

fn dates_grid<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    token_variant: DatePickerTokenVariant,
    month: CalendarMonth,
    month_model: Model<CalendarMonth>,
    week_start: Weekday,
    today: Date,
    selected_now: Option<Date>,
    selected_model: Model<Option<Date>>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let days = month_grid(month, week_start);
    let (
        cell_w,
        cell_h,
        cell_shape,
        label_style,
        unselected_color,
        selected_container,
        selected_label,
        today_outline_width,
        today_outline_color,
        outside_opacity,
    ) = {
        let theme = Theme::global(&*cx.app);
        (
            date_tokens::date_cell_width(theme, token_variant),
            date_tokens::date_cell_height(theme, token_variant),
            date_tokens::date_cell_shape(theme, token_variant),
            date_tokens::date_label_text_style(theme, token_variant),
            date_tokens::date_unselected_label_text_color(theme, token_variant),
            date_tokens::date_selected_container_color(theme, token_variant),
            date_tokens::date_selected_label_text_color(theme, token_variant),
            date_tokens::date_today_outline_width(theme, token_variant),
            date_tokens::date_today_outline_color(theme, token_variant),
            date_tokens::date_outside_month_opacity(theme, token_variant),
        )
    };

    let base_id = test_id.clone().unwrap_or_else(default_date_picker_test_id);

    #[derive(Default)]
    struct DerivedGridTestIds {
        base: Option<Arc<str>>,
        cell_test_ids: Option<Arc<[Arc<str>]>>,
    }

    let cell_test_ids = cx.with_state(DerivedGridTestIds::default, |st| {
        if st.cell_test_ids.is_none() || st.base.as_deref() != Some(base_id.as_ref()) {
            st.base = Some(base_id.clone());
            let mut out: Vec<Arc<str>> = Vec::with_capacity(42);
            for row_idx in 0..6 {
                for col_idx in 0..7 {
                    out.push(Arc::from(format!("{base_id}-cell-{row_idx}-{col_idx}")));
                }
            }
            st.cell_test_ids = Some(Arc::from(out));
        }
        st.cell_test_ids.as_ref().expect("cell_test_ids").clone()
    });

    let mut grid = FlexProps::default();
    grid.direction = Axis::Vertical;
    grid.justify = MainAlign::Start;
    grid.align = CrossAlign::Stretch;
    grid.wrap = false;
    grid.gap = Px(4.0);
    grid.layout.size.width = Length::Fill;

    cx.flex(grid, move |cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        for row_idx in 0..6 {
            let month_model = month_model.clone();
            let selected_model = selected_model.clone();
            let label_style = label_style.clone();
            let cell_test_ids = cell_test_ids.clone();
            let mut row = FlexProps::default();
            row.direction = Axis::Horizontal;
            row.justify = MainAlign::SpaceBetween;
            row.align = CrossAlign::Center;
            row.wrap = false;
            row.gap = Px(0.0);
            row.layout.size.width = Length::Fill;

            let row_days = &days[(row_idx * 7)..((row_idx + 1) * 7)];
            let row_el = cx.flex(row, move |cx| {
                row_days
                    .iter()
                    .enumerate()
                    .map(|(i, day)| {
                        let date = day.date;
                        let in_month = day.in_month;
                        let is_today = date == today;
                        let is_selected = selected_now.is_some_and(|d| d == date);

                        let mut props = ContainerProps::default();
                        props.layout.size.width = Length::Px(cell_w);
                        props.layout.size.height = Length::Px(cell_h);
                        props.corner_radii = cell_shape;
                        props.layout.overflow = Overflow::Clip;

                        if is_selected {
                            props.background = Some(selected_container);
                        }

                        if is_today && !is_selected {
                            props.border = Edges::all(today_outline_width);
                            props.border_color = Some(today_outline_color);
                        }

                        let mut label_props = TextProps::new(cached_day_of_month_label(date.day()));
                        label_props.style = Some(label_style.clone());
                        let mut label_color = if is_selected {
                            selected_label
                        } else {
                            unselected_color
                        };
                        if !in_month && !is_selected {
                            label_color.a = (label_color.a * outside_opacity).clamp(0.0, 1.0);
                        }
                        label_props.color = Some(label_color);
                        label_props.wrap = TextWrap::None;
                        label_props.overflow = TextOverflow::Clip;

                        let cell_test_id = cell_test_ids
                            .get(row_idx * 7 + i)
                            .expect("cell_test_id")
                            .clone();

                        let on_activate: OnActivate = {
                            let selected_model = selected_model.clone();
                            let month_model = month_model.clone();
                            Arc::new(move |host, action_cx, _reason| {
                                let _ = host
                                    .models_mut()
                                    .update(&selected_model, |v| *v = Some(date));
                                if !in_month {
                                    let target = CalendarMonth::from_date(date);
                                    let _ = host.models_mut().update(&month_model, |m| *m = target);
                                }
                                host.request_redraw(action_cx.window);
                            })
                        };

                        cx.pressable(
                            PressableProps {
                                a11y: PressableA11y {
                                    test_id: Some(cell_test_id),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_on_activate(on_activate.clone());
                                vec![
                                    cx.container(props, move |cx| vec![cx.text_props(label_props)]),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            });
            out.push(row_el);
        }
        out
    })
}

fn weekdays_from_start(start: Weekday) -> [Weekday; 7] {
    let all = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];
    let idx = all.iter().position(|w| *w == start).unwrap_or(0);
    std::array::from_fn(|i| all[(idx + i) % 7])
}

fn month_name_en(m: time::Month) -> &'static str {
    use time::Month;
    match m {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    }
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
