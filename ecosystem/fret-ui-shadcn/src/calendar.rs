use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y, PressableProps,
    RovingFlexProps, RovingFocusProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};
use time::{Date, OffsetDateTime, Weekday};

use crate::button::{ButtonSize, ButtonVariant};
use crate::surface_slot::{ShadcnSurfaceSlot, surface_slot_in_scope};

use fret_ui_headless::calendar::{CalendarMonth, month_grid_compact, week_number};
use time::Month;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarLocale {
    En,
    Es,
}

impl Default for CalendarLocale {
    fn default() -> Self {
        Self::En
    }
}

impl CalendarLocale {
    pub(crate) fn month_name(self, month: Month) -> &'static str {
        match self {
            Self::En => match month {
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
            },
            Self::Es => match month {
                Month::January => "enero",
                Month::February => "febrero",
                Month::March => "marzo",
                Month::April => "abril",
                Month::May => "mayo",
                Month::June => "junio",
                Month::July => "julio",
                Month::August => "agosto",
                Month::September => "septiembre",
                Month::October => "octubre",
                Month::November => "noviembre",
                Month::December => "diciembre",
            },
        }
    }

    pub(crate) fn weekday_name(self, weekday: Weekday) -> &'static str {
        match self {
            Self::En => match weekday {
                Weekday::Monday => "Monday",
                Weekday::Tuesday => "Tuesday",
                Weekday::Wednesday => "Wednesday",
                Weekday::Thursday => "Thursday",
                Weekday::Friday => "Friday",
                Weekday::Saturday => "Saturday",
                Weekday::Sunday => "Sunday",
            },
            Self::Es => match weekday {
                Weekday::Monday => "lunes",
                Weekday::Tuesday => "martes",
                Weekday::Wednesday => "miércoles",
                Weekday::Thursday => "jueves",
                Weekday::Friday => "viernes",
                Weekday::Saturday => "sábado",
                Weekday::Sunday => "domingo",
            },
        }
    }

    pub(crate) fn weekday_short(self, weekday: Weekday) -> &'static str {
        match self {
            Self::En => match weekday {
                Weekday::Monday => "Mo",
                Weekday::Tuesday => "Tu",
                Weekday::Wednesday => "We",
                Weekday::Thursday => "Th",
                Weekday::Friday => "Fr",
                Weekday::Saturday => "Sa",
                Weekday::Sunday => "Su",
            },
            Self::Es => match weekday {
                Weekday::Monday => "lu",
                Weekday::Tuesday => "ma",
                Weekday::Wednesday => "mi",
                Weekday::Thursday => "ju",
                Weekday::Friday => "vi",
                Weekday::Saturday => "sá",
                Weekday::Sunday => "do",
            },
        }
    }

    pub(crate) fn month_title(self, month: Month, year: i32) -> Arc<str> {
        Arc::from(format!("{} {year}", self.month_name(month)))
    }

    pub(crate) fn day_aria_label(self, date: Date, today: bool, selected: bool) -> Arc<str> {
        let selected_suffix = if selected { ", selected" } else { "" };
        let today_prefix = if today { "Today, " } else { "" };

        match self {
            Self::En => {
                let day = date.day();
                Arc::from(format!(
                    "{today_prefix}{}, {} {day}{}, {}{selected_suffix}",
                    self.weekday_name(date.weekday()),
                    self.month_name(date.month()),
                    ordinal_suffix(day),
                    date.year(),
                ))
            }
            Self::Es => Arc::from(format!(
                "{today_prefix}{}, {} de {} de {}{selected_suffix}",
                self.weekday_name(date.weekday()),
                date.day(),
                self.month_name(date.month()),
                date.year(),
            )),
        }
    }
}

#[derive(Clone)]
pub struct Calendar {
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
    number_of_months: usize,
    locale: CalendarLocale,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    disable_navigation: bool,
    week_start: Weekday,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    cell_size: Option<Px>,
    today: Option<Date>,
    disabled: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Calendar")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("number_of_months", &self.number_of_months)
            .field("locale", &self.locale)
            .field("month_bounds", &self.month_bounds)
            .field("disable_navigation", &self.disable_navigation)
            .field("week_start", &self.week_start)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled", &self.disabled.is_some())
            .field("close_on_select", &self.close_on_select.is_some())
            .field("initial_focus_out", &self.initial_focus_out.is_some())
            .finish()
    }
}

impl Calendar {
    pub fn new(month: Model<CalendarMonth>, selected: Model<Option<Date>>) -> Self {
        Self {
            month,
            selected,
            number_of_months: 1,
            locale: CalendarLocale::default(),
            month_bounds: None,
            disable_navigation: false,
            week_start: Weekday::Monday,
            show_outside_days: true,
            disable_outside_days: true,
            show_week_number: false,
            cell_size: None,
            today: None,
            disabled: None,
            close_on_select: None,
            initial_focus_out: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = week_start;
        self
    }

    pub fn number_of_months(mut self, months: usize) -> Self {
        self.number_of_months = months.max(1);
        self
    }

    pub fn locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    pub fn month_bounds(mut self, start: CalendarMonth, end: CalendarMonth) -> Self {
        self.month_bounds = Some(if month_le(start, end) {
            (start, end)
        } else {
            (end, start)
        });
        self
    }

    pub fn disable_navigation(mut self, disable: bool) -> Self {
        self.disable_navigation = disable;
        self
    }

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    pub fn disable_outside_days(mut self, disable: bool) -> Self {
        self.disable_outside_days = disable;
        self
    }

    pub fn show_week_number(mut self, show: bool) -> Self {
        self.show_week_number = show;
        self
    }

    pub fn cell_size(mut self, size: Px) -> Self {
        self.cell_size = Some(size);
        self
    }

    /// Overrides the "today" date for deterministic snapshots and testing.
    ///
    /// This mirrors the upstream DayPicker `today` prop.
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }

    pub fn disabled_by(mut self, f: impl Fn(Date) -> bool + Send + Sync + 'static) -> Self {
        self.disabled = Some(Arc::new(f));
        self
    }

    pub fn close_on_select(mut self, open: Model<bool>) -> Self {
        self.close_on_select = Some(open);
        self
    }

    pub(crate) fn initial_focus_out(
        mut self,
        out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> Self {
        self.initial_focus_out = Some(out);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let month_model = self.month.clone();
        let selected_model = self.selected.clone();
        let number_of_months = self.number_of_months.max(1);
        let locale = self.locale;
        let month_bounds = self.month_bounds;
        let disable_navigation = self.disable_navigation;
        let week_start = self.week_start;
        let show_outside_days = self.show_outside_days;
        let disable_outside_days = self.disable_outside_days;
        let show_week_number = self.show_week_number;
        let disabled_predicate = self.disabled.clone();
        let close_on_select = self.close_on_select.clone();
        let initial_focus_out = self.initial_focus_out.clone();

        let month = cx
            .watch_model(&month_model)
            .copied()
            .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
        let selected = cx.watch_model(&selected_model).copied().flatten();

        let grid = month_grid_compact(month, week_start);
        let today = self
            .today
            .unwrap_or_else(|| OffsetDateTime::now_utc().date());
        let in_bounds = |d: Date| month_bounds.map_or(true, |b| date_in_month_bounds(d, b));

        let mut disabled = Vec::with_capacity(grid.len());
        for day in grid.iter() {
            let mut is_disabled = false;
            if !in_bounds(day.date) {
                is_disabled = true;
            }
            if !day.in_month && (!show_outside_days || disable_outside_days) {
                is_disabled = true;
            }
            if let Some(pred) = disabled_predicate.as_ref() {
                if pred(day.date) {
                    is_disabled = true;
                }
            }
            disabled.push(is_disabled);
        }
        let disabled: Arc<[bool]> = disabled.into();

        let focus_date = {
            let selected_idx = selected.and_then(|d| grid.iter().position(|it| it.date == d));
            let today_idx = grid.iter().position(|it| it.date == today);

            let visible = |idx: usize| {
                grid.get(idx)
                    .is_some_and(|d| (d.in_month || show_outside_days) && in_bounds(d.date))
            };
            let enabled = |idx: usize| !disabled.get(idx).copied().unwrap_or(false);

            selected_idx
                .filter(|&idx| visible(idx) && enabled(idx))
                .and_then(|idx| grid.get(idx).map(|d| d.date))
                .or_else(|| {
                    today_idx
                        .filter(|&idx| visible(idx) && enabled(idx))
                        .and_then(|idx| grid.get(idx).map(|d| d.date))
                })
                .or_else(|| {
                    grid.iter()
                        .enumerate()
                        .find(|(idx, day)| {
                            (day.in_month || show_outside_days)
                                && in_bounds(day.date)
                                && enabled(*idx)
                        })
                        .map(|(_, day)| day.date)
                })
        };

        let title = locale.month_title(month.month, month.year);
        let weekday_labels = weekday_labels(locale, week_start);

        let text_sm_px = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let text_sm_line_height = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        let grid_text_style = TextStyle {
            font: Default::default(),
            size: text_sm_px,
            weight: FontWeight::MEDIUM,
            line_height: Some(text_sm_line_height),
            ..Default::default()
        };

        let day_size = self.cell_size.unwrap_or_else(|| {
            theme
                .metric_by_key("component.calendar.day_size")
                .unwrap_or_else(|| theme.metric_required("component.size.sm.icon_button.size"))
        });
        let week_row_gap = theme
            .metric_by_key("component.calendar.week_row_gap")
            .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
        let day_col_gap = Px(0.0);
        let day_grid_width = Px(day_size.0 * 7.0);
        let month_width = if self.show_week_number {
            Px(day_size.0 * 8.0)
        } else {
            day_grid_width
        };

        let bg = theme.color_required("background");
        let mut chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .p(Space::N3);
        if matches!(
            surface_slot_in_scope(cx),
            Some(ShadcnSurfaceSlot::PopoverContent | ShadcnSurfaceSlot::CardContent)
        ) {
            chrome = chrome.bg(ColorRef::Color(Color::TRANSPARENT));
        }
        let chrome = chrome.merge(self.chrome);
        let root = LayoutRefinement::default().merge(self.layout);

        let container_props = decl_style::container_props(&theme, chrome, root);
        cx.container(container_props, move |cx| {
            if number_of_months > 1 {
                return calendar_multi_month_view(
                    cx,
                    &theme,
                    month,
                    month_model.clone(),
                    selected_model.clone(),
                    number_of_months,
                    locale,
                    month_bounds,
                    disable_navigation,
                    week_start,
                    weekday_labels.clone(),
                    selected,
                    today,
                    show_outside_days,
                    disable_outside_days,
                    show_week_number,
                    day_size,
                    month_width,
                    day_grid_width,
                    week_row_gap,
                    disabled_predicate.clone(),
                    close_on_select.clone(),
                    initial_focus_out.clone(),
                    grid_text_style.clone(),
                );
            }
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    let theme_header = theme.clone();
                    let theme_weekdays = theme.clone();
                    let theme_days_for_days = theme.clone();
                    let theme_days_for_week_numbers = theme.clone();

                    let grid_text_style_weekdays = grid_text_style.clone();
                    let grid_text_style_week_numbers = grid_text_style.clone();

                    let month_model_header = month_model.clone();
                    let month_model_days = month_model.clone();
                    let selected_model = selected_model.clone();
                    let close_on_select = close_on_select.clone();
                    let disabled_predicate = disabled_predicate.clone();

                    let header = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_px(MetricRef::Px(month_width)))
                            .items_center()
                            .justify_between(),
                        move |cx| {
                            let nav_enabled = !disable_navigation;
                            let prev_enabled =
                                nav_enabled && month_bounds.map_or(true, |b| month_lt(b.0, month));
                            let next_enabled = nav_enabled
                                && month_bounds
                                    .map_or(true, |b| month_lt(month, max_start_month(b, 1)));

                            let month_model_prev = month_model_header.clone();
                            let prev = calendar_icon_button(
                                cx,
                                "Go to the Previous Month",
                                ButtonVariant::Ghost,
                                ButtonSize::IconSm,
                                day_size,
                                Arc::from("<"),
                                prev_enabled,
                                move |host| {
                                    if disable_navigation {
                                        return;
                                    }
                                    let _ = host.models_mut().update(&month_model_prev, |m| {
                                        let cand = m.prev_month();
                                        *m = month_bounds
                                            .map_or(cand, |b| clamp_start_month(cand, b, 1));
                                    });
                                },
                            );
                            let month_model_next = month_model_header.clone();
                            let next = calendar_icon_button(
                                cx,
                                "Go to the Next Month",
                                ButtonVariant::Ghost,
                                ButtonSize::IconSm,
                                day_size,
                                Arc::from(">"),
                                next_enabled,
                                move |host| {
                                    if disable_navigation {
                                        return;
                                    }
                                    let _ = host.models_mut().update(&month_model_next, |m| {
                                        let cand = m.next_month();
                                        *m = month_bounds
                                            .map_or(cand, |b| clamp_start_month(cand, b, 1));
                                    });
                                },
                            );

                            let mut title_props = TextProps::new(title.clone());
                            title_props.style = Some(TextStyle {
                                font: Default::default(),
                                size: theme_header.metric_required("font.size"),
                                weight: FontWeight::MEDIUM,
                                line_height: Some(theme_header.metric_required("font.line_height")),
                                ..Default::default()
                            });
                            title_props.wrap = TextWrap::None;
                            title_props.overflow = TextOverflow::Clip;
                            let title_el = cx.text_props(title_props);

                            vec![prev, title_el, next]
                        },
                    );

                    let weekday_row = cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: Length::Px(month_width),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Horizontal,
                            gap: day_col_gap,
                            padding: fret_core::Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            let mut out = Vec::with_capacity(8);
                            if show_week_number {
                                let mut props = TextProps::new(Arc::from("Wk"));
                                props.style = Some(grid_text_style_weekdays.clone());
                                props.color = theme_weekdays.color_by_key("muted-foreground");
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;

                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(day_size);
                                layout.size.height = Length::Auto;
                                props.layout = layout;
                                out.push(cx.text_props(props));
                            }

                            out.extend(weekday_labels.iter().map(|label| {
                                let mut props = TextProps::new(Arc::clone(label));
                                props.style = Some(grid_text_style_weekdays.clone());
                                props.color = theme_weekdays.color_by_key("muted-foreground");
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;

                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(day_size);
                                layout.size.height = Length::Auto;
                                props.layout = layout;
                                cx.text_props(props)
                            }));
                            out
                        },
                    );

                    let roving_props = RovingFlexProps {
                        flex: FlexProps {
                            layout: LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: Length::Px(day_grid_width),
                                    ..Default::default()
                                },
                                overflow: Overflow::Visible,
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Horizontal,
                            gap: day_col_gap,
                            padding: fret_core::Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: fret_ui::element::CrossAlign::Start,
                            wrap: true,
                        },
                        roving: RovingFocusProps {
                            enabled: true,
                            wrap: false,
                            disabled: Arc::clone(&disabled),
                        },
                    };

                    let week_numbers: Arc<[u32]> = if show_week_number {
                        grid.chunks(7)
                            .map(|week| week_number(week[0].date, week_start))
                            .collect::<Vec<_>>()
                            .into()
                    } else {
                        Vec::<u32>::new().into()
                    };

                    let days_grid = cx.roving_flex(roving_props, move |cx| {
                        let month_model = month_model_days.clone();
                        cx.roving_on_navigate(Arc::new(move |host, _cx, it| {
                            use fret_core::KeyCode;
                            use fret_ui::action::RovingNavigateResult;

                            let Some(current) = it.current else {
                                return RovingNavigateResult::NotHandled;
                            };

                            let step = match it.key {
                                KeyCode::ArrowLeft => Some(-1),
                                KeyCode::ArrowRight => Some(1),
                                KeyCode::ArrowUp => Some(-7),
                                KeyCode::ArrowDown => Some(7),
                                _ => None,
                            };

                            if let Some(step) = step {
                                let next = (current as i32 + step)
                                    .clamp(0, (it.len.saturating_sub(1)) as i32)
                                    as usize;
                                return RovingNavigateResult::Handled { target: Some(next) };
                            }

                            match it.key {
                                KeyCode::Home => {
                                    let row_start = (current / 7) * 7;
                                    RovingNavigateResult::Handled {
                                        target: Some(row_start),
                                    }
                                }
                                KeyCode::End => {
                                    let row_start = (current / 7) * 7;
                                    let row_end = (row_start + 6).min(it.len.saturating_sub(1));
                                    RovingNavigateResult::Handled {
                                        target: Some(row_end),
                                    }
                                }
                                KeyCode::PageUp => {
                                    let _ = host.models_mut().update(&month_model, |m| {
                                        *m = m.prev_month();
                                    });
                                    RovingNavigateResult::Handled {
                                        target: Some(current),
                                    }
                                }
                                KeyCode::PageDown => {
                                    let _ = host.models_mut().update(&month_model, |m| {
                                        *m = m.next_month();
                                    });
                                    RovingNavigateResult::Handled {
                                        target: Some(current),
                                    }
                                }
                                _ => RovingNavigateResult::NotHandled,
                            }
                        }));

                        grid.iter()
                            .enumerate()
                            .map(|(idx, day)| {
                                let is_hidden =
                                    (!day.in_month && !show_outside_days) || !in_bounds(day.date);
                                if is_hidden {
                                    return calendar_hidden_day_cell(
                                        cx,
                                        &theme_days_for_days,
                                        day_size,
                                        week_row_gap,
                                    );
                                }

                                let is_selected = selected.is_some_and(|d| d == day.date);
                                let is_today = today == day.date;
                                let is_disabled = disabled.get(idx).copied().unwrap_or(false);

                                calendar_day_cell(
                                    cx,
                                    &theme_days_for_days,
                                    locale,
                                    day.date,
                                    day.in_month,
                                    is_selected,
                                    is_today,
                                    is_disabled,
                                    focus_date.is_some_and(|d| d == day.date),
                                    day_size,
                                    week_row_gap,
                                    &selected_model,
                                    close_on_select.clone(),
                                    disabled_predicate.clone(),
                                    initial_focus_out.clone(),
                                )
                            })
                            .collect::<Vec<_>>()
                    });

                    let days = if show_week_number {
                        let week_numbers: Arc<[u32]> = Arc::clone(&week_numbers);
                        let week_number_column = cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: Length::Px(day_size),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0),
                                padding: fret_core::Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Start,
                                wrap: false,
                            },
                            move |cx| {
                                week_numbers
                                    .iter()
                                    .map(|n: &u32| {
                                        let mut props = TextProps::new(Arc::from(n.to_string()));
                                        props.style = Some(grid_text_style_week_numbers.clone());
                                        props.color = theme_days_for_week_numbers
                                            .color_by_key("muted-foreground");
                                        props.wrap = TextWrap::None;
                                        props.overflow = TextOverflow::Clip;

                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(day_size);
                                        layout.size.height = Length::Px(day_size);
                                        layout.margin.bottom =
                                            fret_ui::element::MarginEdge::Px(week_row_gap);
                                        props.layout = layout;
                                        cx.text_props(props)
                                    })
                                    .collect::<Vec<_>>()
                            },
                        );

                        cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: Length::Px(month_width),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: fret_core::Axis::Horizontal,
                                gap: Px(0.0),
                                padding: fret_core::Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Start,
                                wrap: false,
                            },
                            move |_cx| vec![week_number_column, days_grid],
                        )
                    } else {
                        days_grid
                    };

                    let body = stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N2),
                        move |_cx| vec![weekday_row, days],
                    );

                    vec![header, body]
                },
            )]
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn calendar_multi_month_view<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    start_month: CalendarMonth,
    month_model: Model<CalendarMonth>,
    selected_model: Model<Option<Date>>,
    number_of_months: usize,
    locale: CalendarLocale,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    disable_navigation: bool,
    week_start: Weekday,
    weekday_labels: Arc<[Arc<str>]>,
    selected: Option<Date>,
    today: Date,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    day_size: Px,
    month_width: Px,
    day_grid_width: Px,
    week_row_gap: Px,
    disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    grid_text_style: TextStyle,
) -> Vec<AnyElement> {
    let is_row = cx.bounds.size.width.0 >= 768.0;

    let gap_px = decl_style::space(theme, Space::N4);
    let months_span = if is_row {
        Px(month_width.0 * (number_of_months as f32) + gap_px.0 * ((number_of_months - 1) as f32))
    } else {
        month_width
    };

    let mut months = Vec::with_capacity(number_of_months);
    let mut it = start_month;
    for _ in 0..number_of_months {
        months.push(it);
        it = it.next_month();
    }

    let nav = {
        let nav_enabled = !disable_navigation;
        let min_start = month_bounds.map(|b| b.0);
        let max_start = month_bounds.map(|b| max_start_month(b, number_of_months));
        let prev_enabled = nav_enabled && min_start.map_or(true, |min| month_lt(min, start_month));
        let next_enabled = nav_enabled && max_start.map_or(true, |max| month_lt(start_month, max));

        let month_model_prev = month_model.clone();
        let prev = calendar_icon_button(
            cx,
            "Go to the Previous Month",
            ButtonVariant::Ghost,
            ButtonSize::IconSm,
            day_size,
            Arc::from("<"),
            prev_enabled,
            move |host| {
                if disable_navigation {
                    return;
                }
                let _ = host.models_mut().update(&month_model_prev, |m| {
                    let cand = m.prev_month();
                    *m =
                        month_bounds.map_or(cand, |b| clamp_start_month(cand, b, number_of_months));
                });
            },
        );
        let month_model_next = month_model.clone();
        let next = calendar_icon_button(
            cx,
            "Go to the Next Month",
            ButtonVariant::Ghost,
            ButtonSize::IconSm,
            day_size,
            Arc::from(">"),
            next_enabled,
            move |host| {
                if disable_navigation {
                    return;
                }
                let _ = host.models_mut().update(&month_model_next, |m| {
                    let cand = m.next_month();
                    *m =
                        month_bounds.map_or(cand, |b| clamp_start_month(cand, b, number_of_months));
                });
            },
        );

        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Px(months_span);
        layout.position = fret_ui::element::PositionStyle::Absolute;
        layout.inset.top = Some(Px(0.0));
        layout.inset.left = Some(Px(0.0));

        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: decl_style::space(theme, Space::N1),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::SpaceBetween,
                align: fret_ui::element::CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![prev, next],
        )
    };

    let months_el = if is_row {
        ui::h_flex(cx, move |cx| {
            months
                .iter()
                .copied()
                .map(|m| {
                    calendar_month_view(
                        cx,
                        theme,
                        m,
                        locale,
                        month_bounds,
                        week_start,
                        weekday_labels.clone(),
                        selected,
                        today,
                        show_outside_days,
                        disable_outside_days,
                        show_week_number,
                        day_size,
                        month_width,
                        day_grid_width,
                        week_row_gap,
                        month_model.clone(),
                        selected_model.clone(),
                        disabled_predicate.clone(),
                        close_on_select.clone(),
                        initial_focus_out.clone(),
                        grid_text_style.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .gap(Space::N4)
        .items_start()
        .w_px(MetricRef::Px(months_span))
        .into_element(cx)
    } else {
        ui::v_flex(cx, move |cx| {
            months
                .iter()
                .copied()
                .map(|m| {
                    calendar_month_view(
                        cx,
                        theme,
                        m,
                        locale,
                        month_bounds,
                        week_start,
                        weekday_labels.clone(),
                        selected,
                        today,
                        show_outside_days,
                        disable_outside_days,
                        show_week_number,
                        day_size,
                        month_width,
                        day_grid_width,
                        week_row_gap,
                        month_model.clone(),
                        selected_model.clone(),
                        disabled_predicate.clone(),
                        close_on_select.clone(),
                        initial_focus_out.clone(),
                        grid_text_style.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .gap(Space::N4)
        .items_start()
        .w_px(MetricRef::Px(month_width))
        .into_element(cx)
    };

    let stack = ui::stack(cx, move |_cx| vec![months_el, nav])
        .relative()
        .w_px(MetricRef::Px(months_span))
        .into_element(cx);

    vec![stack]
}

#[allow(clippy::too_many_arguments)]
fn calendar_month_view<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    month: CalendarMonth,
    locale: CalendarLocale,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    week_start: Weekday,
    weekday_labels: Arc<[Arc<str>]>,
    selected: Option<Date>,
    today: Date,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    day_size: Px,
    month_width: Px,
    day_grid_width: Px,
    week_row_gap: Px,
    month_model: Model<CalendarMonth>,
    selected_model: Model<Option<Date>>,
    disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    grid_text_style: TextStyle,
) -> AnyElement {
    let grid = month_grid_compact(month, week_start);
    let in_bounds = |d: Date| month_bounds.map_or(true, |b| date_in_month_bounds(d, b));

    let mut disabled = Vec::with_capacity(grid.len());
    for day in grid.iter() {
        let mut is_disabled = false;
        if !in_bounds(day.date) {
            is_disabled = true;
        }
        if !day.in_month && (!show_outside_days || disable_outside_days) {
            is_disabled = true;
        }
        if let Some(pred) = disabled_predicate.as_ref() {
            if pred(day.date) {
                is_disabled = true;
            }
        }
        disabled.push(is_disabled);
    }
    let disabled: Arc<[bool]> = disabled.into();

    let focus_date = {
        let selected_idx = selected.and_then(|d| grid.iter().position(|it| it.date == d));
        let today_idx = grid.iter().position(|it| it.date == today);

        let visible = |idx: usize| {
            grid.get(idx)
                .is_some_and(|d| (d.in_month || show_outside_days) && in_bounds(d.date))
        };
        let enabled = |idx: usize| !disabled.get(idx).copied().unwrap_or(false);

        selected_idx
            .filter(|&idx| visible(idx) && enabled(idx))
            .and_then(|idx| grid.get(idx).map(|d| d.date))
            .or_else(|| {
                today_idx
                    .filter(|&idx| visible(idx) && enabled(idx))
                    .and_then(|idx| grid.get(idx).map(|d| d.date))
            })
            .or_else(|| {
                grid.iter()
                    .enumerate()
                    .find(|(idx, day)| {
                        (day.in_month || show_outside_days) && in_bounds(day.date) && enabled(*idx)
                    })
                    .map(|(_, day)| day.date)
            })
    };

    let title = locale.month_title(month.month, month.year);

    let theme_weekdays = theme.clone();
    let theme_days_for_days = theme.clone();
    let theme_days_for_week_numbers = theme.clone();

    let grid_text_style_caption = grid_text_style.clone();
    let grid_text_style_weekdays = grid_text_style.clone();
    let grid_text_style_week_numbers = grid_text_style.clone();

    let month_caption = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Px(month_width),
                    height: Length::Px(day_size),
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: fret_core::Edges {
                left: day_size,
                right: day_size,
                top: Px(0.0),
                bottom: Px(0.0),
            },
            justify: MainAlign::Center,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let mut props = TextProps::new(title.clone());
            props.style = Some(grid_text_style_caption.clone());
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;
            vec![cx.text_props(props)]
        },
    );

    let weekday_row = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Px(month_width),
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: fret_core::Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::with_capacity(8);
            if show_week_number {
                let mut props = TextProps::new(Arc::from("Wk"));
                props.style = Some(grid_text_style_weekdays.clone());
                props.color = theme_weekdays.color_by_key("muted-foreground");
                props.wrap = TextWrap::None;
                props.overflow = TextOverflow::Clip;

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(day_size);
                layout.size.height = Length::Auto;
                props.layout = layout;
                out.push(cx.text_props(props));
            }

            out.extend(weekday_labels.iter().map(|label| {
                let mut props = TextProps::new(Arc::clone(label));
                props.style = Some(grid_text_style_weekdays.clone());
                props.color = theme_weekdays.color_by_key("muted-foreground");
                props.wrap = TextWrap::None;
                props.overflow = TextOverflow::Clip;

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(day_size);
                layout.size.height = Length::Auto;
                props.layout = layout;
                cx.text_props(props)
            }));
            out
        },
    );

    let roving_props = RovingFlexProps {
        flex: FlexProps {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Px(day_grid_width),
                    ..Default::default()
                },
                overflow: Overflow::Visible,
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: fret_core::Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: fret_ui::element::CrossAlign::Start,
            wrap: true,
        },
        roving: RovingFocusProps {
            enabled: true,
            wrap: false,
            disabled: Arc::clone(&disabled),
        },
    };

    let week_numbers: Arc<[u32]> = if show_week_number {
        grid.chunks(7)
            .map(|week| week_number(week[0].date, week_start))
            .collect::<Vec<_>>()
            .into()
    } else {
        Vec::<u32>::new().into()
    };

    let days_grid = cx.roving_flex(roving_props, move |cx| {
        let month_model = month_model.clone();
        cx.roving_on_navigate(Arc::new(move |host, _cx, it| {
            use fret_core::KeyCode;
            use fret_ui::action::RovingNavigateResult;

            let Some(current) = it.current else {
                return RovingNavigateResult::NotHandled;
            };

            let step = match it.key {
                KeyCode::ArrowLeft => Some(-1),
                KeyCode::ArrowRight => Some(1),
                KeyCode::ArrowUp => Some(-7),
                KeyCode::ArrowDown => Some(7),
                _ => None,
            };

            if let Some(step) = step {
                let next =
                    (current as i32 + step).clamp(0, (it.len.saturating_sub(1)) as i32) as usize;
                return RovingNavigateResult::Handled { target: Some(next) };
            }

            match it.key {
                KeyCode::Home => {
                    let row_start = (current / 7) * 7;
                    RovingNavigateResult::Handled {
                        target: Some(row_start),
                    }
                }
                KeyCode::End => {
                    let row_start = (current / 7) * 7;
                    let row_end = (row_start + 6).min(it.len.saturating_sub(1));
                    RovingNavigateResult::Handled {
                        target: Some(row_end),
                    }
                }
                KeyCode::PageUp => {
                    let _ = host.models_mut().update(&month_model, |m| {
                        *m = m.prev_month();
                    });
                    RovingNavigateResult::Handled {
                        target: Some(current),
                    }
                }
                KeyCode::PageDown => {
                    let _ = host.models_mut().update(&month_model, |m| {
                        *m = m.next_month();
                    });
                    RovingNavigateResult::Handled {
                        target: Some(current),
                    }
                }
                _ => RovingNavigateResult::NotHandled,
            }
        }));

        grid.iter()
            .enumerate()
            .map(|(idx, day)| {
                let is_hidden = (!day.in_month && !show_outside_days) || !in_bounds(day.date);
                if is_hidden {
                    return calendar_hidden_day_cell(
                        cx,
                        &theme_days_for_days,
                        day_size,
                        week_row_gap,
                    );
                }

                let is_selected = selected.is_some_and(|d| d == day.date);
                let is_today = today == day.date;
                let is_disabled = disabled.get(idx).copied().unwrap_or(false);

                calendar_day_cell(
                    cx,
                    &theme_days_for_days,
                    locale,
                    day.date,
                    day.in_month,
                    is_selected,
                    is_today,
                    is_disabled,
                    focus_date.is_some_and(|d| d == day.date),
                    day_size,
                    week_row_gap,
                    &selected_model,
                    close_on_select.clone(),
                    disabled_predicate.clone(),
                    initial_focus_out.clone(),
                )
            })
            .collect::<Vec<_>>()
    });

    let days = if show_week_number {
        let week_numbers: Arc<[u32]> = Arc::clone(&week_numbers);
        let week_number_column = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(day_size),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: fret_ui::element::CrossAlign::Start,
                wrap: false,
            },
            move |cx| {
                week_numbers
                    .iter()
                    .map(|n: &u32| {
                        let mut props = TextProps::new(Arc::from(n.to_string()));
                        props.style = Some(grid_text_style_week_numbers.clone());
                        props.color = theme_days_for_week_numbers.color_by_key("muted-foreground");
                        props.wrap = TextWrap::None;
                        props.overflow = TextOverflow::Clip;

                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(day_size);
                        layout.size.height = Length::Px(day_size);
                        layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);
                        props.layout = layout;
                        cx.text_props(props)
                    })
                    .collect::<Vec<_>>()
            },
        );

        cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(month_width),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: fret_ui::element::CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![week_number_column, days_grid],
        )
    } else {
        days_grid
    };

    let body = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2),
        move |_cx| vec![weekday_row, days],
    );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4),
        move |_cx| vec![month_caption, body],
    )
}

fn weekday_labels(locale: CalendarLocale, week_start: Weekday) -> Arc<[Arc<str>]> {
    let order = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    let start_idx = order.iter().position(|d| *d == week_start).unwrap_or(0);

    let mut out = Vec::with_capacity(7);
    for i in 0..7 {
        let day = order[(start_idx + i) % 7];
        out.push(Arc::from(locale.weekday_short(day)));
    }
    out.into()
}

pub(crate) fn month_key(month: CalendarMonth) -> (i32, u8) {
    (month.year, month_number(month.month))
}

pub(crate) fn date_month_key(date: Date) -> (i32, u8) {
    (date.year(), month_number(date.month()))
}

fn month_number(month: Month) -> u8 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}

pub(crate) fn month_lt(a: CalendarMonth, b: CalendarMonth) -> bool {
    month_key(a) < month_key(b)
}

pub(crate) fn month_le(a: CalendarMonth, b: CalendarMonth) -> bool {
    month_key(a) <= month_key(b)
}

pub(crate) fn date_in_month_bounds(date: Date, bounds: (CalendarMonth, CalendarMonth)) -> bool {
    let k = date_month_key(date);
    k >= month_key(bounds.0) && k <= month_key(bounds.1)
}

pub(crate) fn max_start_month(
    bounds: (CalendarMonth, CalendarMonth),
    number_of_months: usize,
) -> CalendarMonth {
    let mut max_start = bounds.1;
    for _ in 1..number_of_months.max(1) {
        max_start = max_start.prev_month();
    }
    if month_le(max_start, bounds.0) {
        bounds.0
    } else {
        max_start
    }
}

pub(crate) fn clamp_start_month(
    cand: CalendarMonth,
    bounds: (CalendarMonth, CalendarMonth),
    number_of_months: usize,
) -> CalendarMonth {
    let min_start = bounds.0;
    let max_start = max_start_month(bounds, number_of_months);
    if month_key(cand) < month_key(min_start) {
        min_start
    } else if month_key(cand) > month_key(max_start) {
        max_start
    } else {
        cand
    }
}

fn ordinal_suffix(day: u8) -> &'static str {
    let mod_100 = day % 100;
    if mod_100 >= 11 && mod_100 <= 13 {
        return "th";
    }
    match day % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

fn calendar_icon_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    variant: crate::button::ButtonVariant,
    size: ButtonSize,
    button_size_px: Px,
    text: Arc<str>,
    enabled: bool,
    on_activate: impl Fn(&mut dyn fret_ui::action::UiActionHost) + 'static,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let (bg, bg_hover, bg_pressed, border, fg) = crate::button::variant_colors(&theme, variant);

    let radius = theme
        .metric_by_key("component.button.radius")
        .unwrap_or_else(|| theme.metric_required("metric.radius.md"));

    control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            on_activate(host);
        }));

        let mut pressable_layout = LayoutStyle::default();
        pressable_layout.size.width = Length::Px(button_size_px);
        pressable_layout.size.height = Length::Px(button_size_px);

        let bg = if st.pressed {
            bg_pressed
        } else if st.hovered {
            bg_hover
        } else {
            bg
        };

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border));

        let pressable = PressableProps {
            layout: pressable_layout,
            enabled,
            focusable: enabled,
            focus_ring: Some(decl_style::focus_ring(&theme, radius)),
            a11y: PressableA11y {
                label: Some(Arc::from(label)),
                ..Default::default()
            },
            ..Default::default()
        };

        let chrome_props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());

        let style = crate::button::button_text_style(&theme, size);
        let children = move |cx: &mut ElementContext<'_, H>| {
            let mut label = ui::label(cx, text.clone())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(fg))
                .nowrap();
            if let Some(line_height) = style.line_height {
                label = label.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = style.letter_spacing_em {
                label = label.letter_spacing_em(letter_spacing_em);
            }
            vec![label.into_element(cx)]
        };

        (pressable, chrome_props, children)
    })
}

fn calendar_hidden_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    size: Px,
    week_row_gap: Px,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);

    control_chrome_pressable_with_id_props(cx, move |_cx, _st, _id| {
        let mut chrome_props = decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default(),
        );
        chrome_props.layout.margin = Default::default();

        let pressable = PressableProps {
            layout,
            enabled: false,
            focusable: false,
            focus_ring: None,
            a11y: PressableA11y {
                hidden: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let children = move |_cx: &mut ElementContext<'_, H>| Vec::new();
        (pressable, chrome_props, children)
    })
}

fn calendar_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    locale: CalendarLocale,
    date: Date,
    in_month: bool,
    selected: bool,
    today: bool,
    disabled: bool,
    focus_candidate: bool,
    size: Px,
    week_row_gap: Px,
    selected_model: &Model<Option<Date>>,
    close_on_select: Option<Model<bool>>,
    disabled_predicate: Option<Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);

    let muted_fg = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let fg = if in_month {
        theme.color_required("foreground")
    } else {
        muted_fg
    };

    let (bg, fg) = if selected {
        (
            theme.color_required("primary"),
            theme.color_required("primary-foreground"),
        )
    } else {
        (Color::TRANSPARENT, fg)
    };

    let ring_color = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"));

    let day = date.day();
    let day_text: Arc<str> = Arc::from(day.to_string());
    let date_label = locale.day_aria_label(date, today, selected);

    let text_sm_px = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let text_sm_line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
        if focus_candidate
            && !disabled
            && let Some(out) = initial_focus_out.as_ref()
            && out.get().is_none()
        {
            out.set(Some(id));
        }

        let selected_model = selected_model.clone();
        let close_on_select = close_on_select.clone();
        let disabled_predicate = disabled_predicate.clone();

        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            if disabled {
                return;
            }
            if let Some(pred) = disabled_predicate.as_ref()
                && pred(date)
            {
                return;
            }
            let _ = host
                .models_mut()
                .update(&selected_model, |v| *v = Some(date));
            if let Some(open) = close_on_select.as_ref() {
                let _ = host.models_mut().update(open, |v| *v = false);
            }
        }));

        let hover_bg = theme.color_required("accent");
        let pressed_bg = {
            let mut c = hover_bg;
            c.a *= 0.85;
            c
        };

        let bg = if selected {
            bg
        } else if st.pressed {
            pressed_bg
        } else if st.hovered {
            hover_bg
        } else {
            Color::TRANSPARENT
        };

        let mut chrome = ChromeRefinement::default()
            .rounded(Radius::Sm)
            .bg(ColorRef::Color(bg));
        if today && !selected {
            chrome = chrome.border_1().border_color(ColorRef::Color(ring_color));
        }

        let mut chrome_props =
            decl_style::container_props(theme, chrome, LayoutRefinement::default());
        // Margins are outside the control box (CSS mental model). Keep them on the pressable node
        // so row gaps don't inflate the chrome/background quad.
        chrome_props.layout.margin = Default::default();

        let pressable = PressableProps {
            layout,
            enabled: !disabled,
            focusable: !disabled,
            focus_ring: Some(decl_style::focus_ring(
                theme,
                theme.metric_required("metric.radius.sm"),
            )),
            a11y: PressableA11y {
                label: Some(date_label.clone()),
                test_id: Some(Arc::from(date.to_string())),
                selected,
                ..Default::default()
            },
            ..Default::default()
        };

        let children = move |cx: &mut ElementContext<'_, H>| {
            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: fret_core::Axis::Vertical,
                    gap: Px(0.0),
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    vec![
                        ui::label(cx, day_text.clone())
                            .text_size_px(text_sm_px)
                            .line_height_px(text_sm_line_height)
                            .font_medium()
                            .text_color(ColorRef::Color(if disabled { muted_fg } else { fg }))
                            .nowrap()
                            .into_element(cx),
                    ]
                },
            )]
        };

        (pressable, chrome_props, children)
    })
}
