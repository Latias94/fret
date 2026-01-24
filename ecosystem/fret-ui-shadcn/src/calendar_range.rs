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

use fret_ui_headless::calendar::{
    CalendarMonth, DateRangeSelection, month_grid_compact, week_number,
};

#[derive(Clone)]
pub struct CalendarRange {
    month: Model<CalendarMonth>,
    selected: Model<DateRangeSelection>,
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

impl std::fmt::Debug for CalendarRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalendarRange")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("week_start", &self.week_start)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled", &self.disabled.is_some())
            .field("close_on_select", &self.close_on_select.is_some())
            .field("initial_focus_out", &self.initial_focus_out.is_some())
            .finish()
    }
}

impl CalendarRange {
    pub fn new(month: Model<CalendarMonth>, selected: Model<DateRangeSelection>) -> Self {
        Self {
            month,
            selected,
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

    /// Closes the parent popover when the selection becomes complete (both ends chosen).
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
        let disabled_predicate = self.disabled.clone();
        let close_on_select = self.close_on_select.clone();
        let initial_focus_out = self.initial_focus_out.clone();

        let month = cx
            .watch_model(&month_model)
            .copied()
            .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
        let selected = cx.watch_model(&selected_model).cloned().unwrap_or_default();

        let grid = month_grid_compact(month, self.week_start);
        let today = self
            .today
            .unwrap_or_else(|| OffsetDateTime::now_utc().date());

        let mut disabled = Vec::with_capacity(grid.len());
        for day in grid.iter() {
            let mut is_disabled = false;
            if !day.in_month && (!self.show_outside_days || self.disable_outside_days) {
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
            let preferred = selected.from.or(selected.to);
            let preferred_idx = preferred.and_then(|d| grid.iter().position(|it| it.date == d));
            let today_idx = grid.iter().position(|it| it.date == today);

            let visible = |idx: usize| {
                grid.get(idx)
                    .is_some_and(|d| d.in_month || self.show_outside_days)
            };
            let enabled = |idx: usize| !disabled.get(idx).copied().unwrap_or(false);

            preferred_idx
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
                            (day.in_month || self.show_outside_days) && enabled(*idx)
                        })
                        .map(|(_, day)| day.date)
                })
        };

        let title: Arc<str> = Arc::from(format!("{:?} {}", month.month, month.year));
        let weekday_labels = weekday_labels(self.week_start);
        let show_week_number = self.show_week_number;

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

        let chrome = ChromeRefinement::default().p(Space::N3).merge(self.chrome);
        let root = LayoutRefinement::default().w_full().merge(self.layout);
        let container_props = decl_style::container_props(&theme, chrome, root);

        cx.container(container_props, move |cx| {
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
                            let month_model_prev = month_model_header.clone();
                            let prev = calendar_icon_button(
                                cx,
                                "Go to the Previous Month",
                                ButtonVariant::Ghost,
                                ButtonSize::IconSm,
                                day_size,
                                Arc::from("<"),
                                move |host| {
                                    let _ = host.models_mut().update(&month_model_prev, |m| {
                                        *m = m.prev_month();
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
                                move |host| {
                                    let _ = host.models_mut().update(&month_model_next, |m| {
                                        *m = m.next_month();
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
                            let title = cx.text_props(title_props);

                            vec![prev, title, next]
                        },
                    );

                    let weekday_row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_px(MetricRef::Px(month_width))),
                        move |cx| {
                            let mut out = Vec::with_capacity(8);
                            if show_week_number {
                                let mut props = TextProps::new(Arc::from("Wk"));
                                props.style = Some(grid_text_style_weekdays.clone());
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                props.color =
                                    Some(theme_weekdays.color_required("muted-foreground"));
                                props.layout = {
                                    let mut ls = LayoutStyle::default();
                                    ls.size.width = Length::Px(day_size);
                                    ls.size.height = Length::Auto;
                                    ls
                                };
                                out.push(cx.text_props(props));
                            }

                            out.extend(weekday_labels.iter().map(|label| {
                                let mut props = TextProps::new(label.clone());
                                props.style = Some(grid_text_style_weekdays.clone());
                                props.wrap = TextWrap::None;
                                props.overflow = TextOverflow::Clip;
                                props.color =
                                    Some(theme_weekdays.color_required("muted-foreground"));
                                props.layout = {
                                    let mut ls = LayoutStyle::default();
                                    ls.size.width = Length::Px(day_size);
                                    ls.size.height = Length::Auto;
                                    ls
                                };
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
                            .map(|week| week_number(week[0].date, self.week_start))
                            .collect::<Vec<_>>()
                            .into()
                    } else {
                        Arc::from([])
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
                                let is_hidden = !day.in_month && !self.show_outside_days;
                                if is_hidden {
                                    return calendar_range_hidden_day_cell(
                                        cx,
                                        &theme_days_for_days,
                                        day_size,
                                        week_row_gap,
                                    );
                                }

                                let is_today = today == day.date;
                                let is_disabled = disabled.get(idx).copied().unwrap_or(false);

                                let is_from = selected.from.is_some_and(|d| d == day.date);
                                let is_to = selected.to.is_some_and(|d| d == day.date);
                                let in_range = selected.contains(day.date);
                                let selected_flag = in_range || is_from || is_to;

                                calendar_range_day_cell(
                                    cx,
                                    &theme_days_for_days,
                                    day.date,
                                    day.in_month,
                                    is_from,
                                    is_to,
                                    selected_flag,
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
                        let week_numbers = Arc::clone(&week_numbers);
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
                                    .map(|n| {
                                        let mut props = TextProps::new(Arc::from(n.to_string()));
                                        props.style = Some(grid_text_style_week_numbers.clone());
                                        props.color = Some(
                                            theme_days_for_week_numbers
                                                .color_required("muted-foreground"),
                                        );
                                        props.wrap = TextWrap::None;
                                        props.overflow = TextOverflow::Clip;
                                        props.layout = {
                                            let mut ls = LayoutStyle::default();
                                            ls.size.width = Length::Px(day_size);
                                            ls.size.height = Length::Px(day_size);
                                            ls.margin.bottom =
                                                fret_ui::element::MarginEdge::Px(week_row_gap);
                                            ls
                                        };
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

fn weekday_labels(week_start: Weekday) -> Arc<[Arc<str>]> {
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
        out.push(Arc::from(match day {
            Weekday::Monday => "Mon",
            Weekday::Tuesday => "Tue",
            Weekday::Wednesday => "Wed",
            Weekday::Thursday => "Thu",
            Weekday::Friday => "Fri",
            Weekday::Saturday => "Sat",
            Weekday::Sunday => "Sun",
        }));
    }
    out.into()
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
            enabled: true,
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

fn calendar_range_hidden_day_cell<H: UiHost>(
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
        chrome_props.layout = layout;

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

#[allow(clippy::too_many_arguments)]
fn calendar_range_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    date: Date,
    in_month: bool,
    is_from: bool,
    is_to: bool,
    selected: bool,
    today: bool,
    disabled: bool,
    focus_candidate: bool,
    size: Px,
    week_row_gap: Px,
    selected_model: &Model<DateRangeSelection>,
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

    let (bg, fg) = if is_from || is_to {
        (
            theme.color_required("primary"),
            theme.color_required("primary-foreground"),
        )
    } else if selected {
        (theme.color_required("accent"), fg)
    } else {
        (Color::TRANSPARENT, fg)
    };

    let ring_color = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"));

    let day = date.day();
    let day_text: Arc<str> = Arc::from(day.to_string());
    let selected_suffix = if selected { ", selected" } else { "" };
    let today_prefix = if today { "Today, " } else { "" };
    let date_label: Arc<str> = Arc::from(format!(
        "{today_prefix}{:?}, {:?} {day}{}, {}{selected_suffix}",
        date.weekday(),
        date.month(),
        ordinal_suffix(day),
        date.year()
    ));

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

        let complete_out = Rc::new(Cell::new(false));
        let complete_out_for_update = Rc::clone(&complete_out);

        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            if disabled {
                return;
            }
            if let Some(pred) = disabled_predicate.as_ref()
                && pred(date)
            {
                return;
            }

            complete_out_for_update.set(false);
            let complete_out_in_update = Rc::clone(&complete_out_for_update);
            let _ = host.models_mut().update(&selected_model, |v| {
                v.apply_click(date);
                complete_out_in_update.set(v.is_complete());
            });

            if complete_out_for_update.get()
                && let Some(open) = close_on_select.as_ref()
            {
                let _ = host.models_mut().update(open, |v| *v = false);
            }
        }));

        let hover_bg = theme.color_required("accent");
        let pressed_bg = {
            let mut c = hover_bg;
            c.a *= 0.85;
            c
        };

        let bg = if is_from || is_to {
            bg
        } else if st.pressed {
            pressed_bg
        } else if st.hovered {
            hover_bg
        } else {
            bg
        };

        let mut chrome = ChromeRefinement::default()
            .rounded(Radius::Sm)
            .bg(ColorRef::Color(bg));
        if today && !(is_from || is_to) {
            chrome = chrome.border_1().border_color(ColorRef::Color(ring_color));
        }

        let mut chrome_props =
            decl_style::container_props(theme, chrome, LayoutRefinement::default());
        chrome_props.layout = layout;

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
                selected,
                ..Default::default()
            },
            ..Default::default()
        };

        let children = move |cx: &mut ElementContext<'_, H>| {
            vec![
                ui::label(cx, day_text.clone())
                    .text_size_px(text_sm_px)
                    .line_height_px(text_sm_line_height)
                    .font_medium()
                    .text_color(ColorRef::Color(if disabled { muted_fg } else { fg }))
                    .nowrap()
                    .into_element(cx),
            ]
        };

        (pressable, chrome_props, children)
    })
}
