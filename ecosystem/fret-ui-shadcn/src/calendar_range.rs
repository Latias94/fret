use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FlexProps, LayoutQueryRegionProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Corners4, LayoutRefinement, MetricRef, Radius, Space, ui,
};
use time::{Date, OffsetDateTime, Weekday};

use crate::bool_model::IntoBoolModel;
use crate::calendar::{
    CalendarCaptionLayout, CalendarDayButton, CalendarDayButtonInfo, CalendarLocale,
    CalendarMultiMonthNavProps, CalendarSingleMonthHeaderProps, calendar_caption_value_models,
    calendar_day_button_children, calendar_day_button_supporting_text,
    calendar_day_grid_row_edge_target_for_key, calendar_day_grid_step_for_key,
    calendar_month_caption, calendar_multi_month_nav_overlay, calendar_single_month_header,
    calendar_weekday_row,
};
use crate::calendar_month_model::IntoCalendarMonthModel;
use crate::date_range_selection_model::IntoDateRangeSelectionModel;
use crate::surface_slot::{ShadcnSurfaceSlot, surface_slot_in_scope};

use fret_ui_headless::calendar::{
    CalendarMonth, DateRangeSelection, DayMatcher, DayPickerModifiers,
    day_grid_row_edge_target_skipping_disabled, day_grid_step_target_skipping_disabled,
    day_picker_cell_state, month_grid, month_grid_compact, week_number,
};

#[derive(Clone)]
pub struct CalendarRange {
    month: Model<CalendarMonth>,
    selected: Model<DateRangeSelection>,
    number_of_months: usize,
    locale: CalendarLocale,
    caption_layout: CalendarCaptionLayout,
    test_id_prefix: Option<Arc<str>>,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    disable_navigation: bool,
    required: bool,
    min_days: i64,
    max_days: i64,
    exclude_disabled: bool,
    week_start: Weekday,
    fixed_weeks: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    cell_size: Option<Px>,
    today: Option<Date>,
    modifiers: DayPickerModifiers,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    day_button: CalendarDayButton,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CalendarRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalendarRange")
            .field("month", &"<model>")
            .field("selected", &"<model>")
            .field("number_of_months", &self.number_of_months)
            .field("locale", &self.locale)
            .field("caption_layout", &self.caption_layout)
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .field("month_bounds", &self.month_bounds)
            .field("disable_navigation", &self.disable_navigation)
            .field("required", &self.required)
            .field("min_days", &self.min_days)
            .field("max_days", &self.max_days)
            .field("exclude_disabled", &self.exclude_disabled)
            .field("week_start", &self.week_start)
            .field("fixed_weeks", &self.fixed_weeks)
            .field("show_outside_days", &self.show_outside_days)
            .field("disable_outside_days", &self.disable_outside_days)
            .field("disabled_matchers", &self.modifiers.disabled.len())
            .field("hidden_matchers", &self.modifiers.hidden.len())
            .field("close_on_select", &self.close_on_select.is_some())
            .field("initial_focus_out", &self.initial_focus_out.is_some())
            .field("day_button", &self.day_button)
            .finish()
    }
}

impl CalendarRange {
    pub fn new(
        month: impl IntoCalendarMonthModel,
        selected: impl IntoDateRangeSelectionModel,
    ) -> Self {
        Self {
            month: month.into_calendar_month_model(),
            selected: selected.into_date_range_selection_model(),
            number_of_months: 1,
            locale: CalendarLocale::default(),
            caption_layout: CalendarCaptionLayout::default(),
            test_id_prefix: None,
            month_bounds: None,
            disable_navigation: false,
            required: false,
            min_days: 0,
            max_days: 0,
            exclude_disabled: false,
            week_start: Weekday::Sunday,
            fixed_weeks: false,
            show_outside_days: true,
            disable_outside_days: false,
            show_week_number: false,
            cell_size: None,
            today: None,
            modifiers: DayPickerModifiers::default(),
            close_on_select: None,
            initial_focus_out: None,
            day_button: CalendarDayButton::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn week_start(mut self, week_start: Weekday) -> Self {
        self.week_start = week_start;
        self
    }

    /// Mirrors the upstream DayPicker `fixedWeeks` prop.
    ///
    /// When enabled, the calendar grid always contains 6 weeks (42 days).
    pub fn fixed_weeks(mut self, fixed: bool) -> Self {
        self.fixed_weeks = fixed;
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

    pub fn caption_layout(mut self, caption_layout: CalendarCaptionLayout) -> Self {
        self.caption_layout = caption_layout;
        self
    }

    /// Sets a stable `test_id` prefix for calendar parts (days, etc.).
    ///
    /// When unset, day cells default to `YYYY-MM-DD`.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn month_bounds(mut self, start: CalendarMonth, end: CalendarMonth) -> Self {
        self.month_bounds = Some(if crate::calendar::month_le(start, end) {
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

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn min_days(mut self, min_days: i64) -> Self {
        self.min_days = min_days.max(0);
        self
    }

    pub fn max_days(mut self, max_days: i64) -> Self {
        self.max_days = max_days.max(0);
        self
    }

    pub fn exclude_disabled(mut self, exclude_disabled: bool) -> Self {
        self.exclude_disabled = exclude_disabled;
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

    pub fn disabled(mut self, matcher: DayMatcher) -> Self {
        self.modifiers.disabled.push(matcher);
        self
    }

    pub fn hidden(mut self, matcher: DayMatcher) -> Self {
        self.modifiers.hidden.push(matcher);
        self
    }

    pub fn modifiers(mut self, modifiers: DayPickerModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn disabled_by(mut self, f: impl Fn(Date) -> bool + Send + Sync + 'static) -> Self {
        self.modifiers.disabled.clear();
        self.modifiers
            .disabled
            .push(DayMatcher::Predicate(Arc::new(f)));
        self
    }

    /// Closes the parent popover when the selection becomes complete (both ends chosen).
    pub fn close_on_select(mut self, open: impl IntoBoolModel) -> Self {
        self.close_on_select = Some(open.into_bool_model());
        self
    }

    pub fn day_button(mut self, day_button: CalendarDayButton) -> Self {
        self.day_button = day_button;
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let month_model = self.month.clone();
        let selected_model = self.selected.clone();
        let number_of_months = self.number_of_months.max(1);
        let locale = self.locale;
        let caption_layout = self.caption_layout;
        let test_id_prefix = self.test_id_prefix.clone();
        let month_bounds = self.month_bounds;
        let disable_navigation = self.disable_navigation;
        let required = self.required;
        let min_days = self.min_days;
        let max_days = self.max_days;
        let exclude_disabled = self.exclude_disabled;
        let week_start = self.week_start;
        let fixed_weeks = self.fixed_weeks;
        let show_outside_days = self.show_outside_days;
        let disable_outside_days = self.disable_outside_days;
        let show_week_number = self.show_week_number;
        let modifiers: Arc<DayPickerModifiers> = Arc::new(self.modifiers);
        let close_on_select = self.close_on_select.clone();
        let initial_focus_out = self.initial_focus_out.clone();
        let day_button = self.day_button.clone();

        let month = cx
            .watch_model(&month_model)
            .copied()
            .unwrap_or_else(|| CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
        let selected = cx.watch_model(&selected_model).cloned().unwrap_or_default();

        let grid = if fixed_weeks {
            month_grid(month, week_start).to_vec()
        } else {
            month_grid_compact(month, week_start)
        };
        let today = self
            .today
            .unwrap_or_else(|| OffsetDateTime::now_utc().date());
        let in_bounds =
            |d: Date| month_bounds.map_or(true, |b| crate::calendar::date_in_month_bounds(d, b));

        let mut hidden = Vec::with_capacity(grid.len());
        let mut disabled = Vec::with_capacity(grid.len());
        for day in grid.iter() {
            let st = day_picker_cell_state(
                *day,
                show_outside_days,
                disable_outside_days,
                in_bounds(day.date),
                modifiers.as_ref(),
            );
            hidden.push(st.hidden);
            disabled.push(st.disabled);
        }
        let disabled: Arc<[bool]> = disabled.into();
        let hidden: Arc<[bool]> = hidden.into();

        let focus_date = {
            let preferred = selected.from.or(selected.to);
            let preferred_idx = preferred.and_then(|d| grid.iter().position(|it| it.date == d));
            let today_idx = grid.iter().position(|it| it.date == today);

            let visible =
                |idx: usize| !hidden.get(idx).copied().unwrap_or(true) && grid.get(idx).is_some();
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
                        .find(|(idx, _day)| visible(*idx) && enabled(*idx))
                        .and_then(|(idx, _day)| grid.get(idx).map(|d| d.date))
                })
        };

        let title = locale.month_title(month.month, month.year);
        let weekday_labels = weekday_labels(locale, week_start);

        let text_sm_px = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let text_sm_line_height = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let mut grid_text_style = typography::fixed_line_box_style(
            fret_core::FontId::ui(),
            text_sm_px,
            text_sm_line_height,
        );
        grid_text_style.weight = FontWeight::MEDIUM;

        let day_size = self.cell_size.unwrap_or_else(|| {
            theme
                .metric_by_key("component.calendar.day_size")
                .unwrap_or_else(|| theme.metric_token("component.size.sm.icon_button.size"))
        });
        let week_row_gap = theme
            .metric_by_key("component.calendar.week_row_gap")
            .unwrap_or_else(|| theme.metric_token("metric.padding.sm"));
        let day_col_gap = Px(0.0);
        let day_grid_width = Px(day_size.0 * 7.0);
        let month_width = if show_week_number {
            Px(day_size.0 * 8.0)
        } else {
            day_grid_width
        };

        let chrome_override = self.chrome;
        let layout_override = self.layout;

        let region_props = LayoutQueryRegionProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            name: None,
        };

        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "shadcn.calendar_range",
            region_props,
            move |cx, region_id| {
                let is_row = if number_of_months > 1 {
                    if matches!(
                        surface_slot_in_scope(cx),
                        Some(ShadcnSurfaceSlot::PopoverContent)
                    ) {
                        cx.environment_viewport_width(Invalidation::Layout).0
                            >= fret_ui_kit::declarative::container_queries::tailwind::MD.0
                    } else {
                        // Container queries are read from last-committed bounds. In single-pass layout
                        // environments (e.g. snapshot tests), the region width can be temporarily
                        // unknown. Fall back to the viewport width so the initial layout matches the
                        // web Tailwind breakpoint behavior when the calendar is effectively
                        // unconstrained by a smaller container.
                        let default_when_unknown =
                            cx.environment_viewport_width(Invalidation::Layout).0
                                >= fret_ui_kit::declarative::container_queries::tailwind::MD.0;
                        fret_ui_kit::declarative::container_width_at_least(
                            cx,
                            region_id,
                            Invalidation::Layout,
                            default_when_unknown,
                            fret_ui_kit::declarative::container_queries::tailwind::MD,
                            fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                        )
                    }
                } else {
                    false
                };

                let bg = theme.color_token("background");
                let mut chrome = ChromeRefinement::default()
                    .bg(ColorRef::Color(bg))
                    .p(Space::N3);
                if matches!(
                    surface_slot_in_scope(cx),
                    Some(ShadcnSurfaceSlot::PopoverContent | ShadcnSurfaceSlot::CardContent)
                ) {
                    chrome = chrome.bg(ColorRef::Color(Color::TRANSPARENT));
                }
                let chrome = chrome.merge(chrome_override);
                let root = LayoutRefinement::default().merge(layout_override);
                let container_props = decl_style::container_props(&theme, chrome, root);

                let (caption_month_value, caption_year_value) =
                    calendar_caption_value_models(cx, caption_layout, month);

                vec![cx.container(container_props, move |cx| {
                    if number_of_months > 1 {
                        return calendar_range_multi_month_view(
                            cx,
                            &theme,
                            is_row,
                            month,
                            month_model.clone(),
                            selected_model.clone(),
                            required,
                            min_days,
                            max_days,
                            exclude_disabled,
                            number_of_months,
                            locale,
                            test_id_prefix.clone(),
                            month_bounds,
                            disable_navigation,
                            week_start,
                            fixed_weeks,
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
                            modifiers.clone(),
                            close_on_select.clone(),
                            initial_focus_out.clone(),
                            grid_text_style.clone(),
                            day_button.clone(),
                        );
                    }
                    vec![
                        ui::v_stack(move |cx| {
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
                            let modifiers = modifiers.clone();
                            let day_button = day_button.clone();
                            let test_id_prefix_for_header = test_id_prefix.clone();

                            let header = calendar_single_month_header(
                                cx,
                                "shadcn.calendar_range.header",
                                CalendarSingleMonthHeaderProps {
                                    theme: theme_header,
                                    locale,
                                    title: title.clone(),
                                    month,
                                    month_model: month_model_header,
                                    caption_layout,
                                    caption_values: (
                                        caption_month_value.clone(),
                                        caption_year_value.clone(),
                                    ),
                                    test_id_prefix: test_id_prefix_for_header,
                                    day_size,
                                    month_width,
                                    month_bounds,
                                    disable_navigation,
                                    today,
                                },
                            );

                            let weekday_row = ui::h_row(move |cx| {
                                let mut out = Vec::with_capacity(8);
                                if show_week_number {
                                    let mut props = TextProps::new(Arc::from("Wk"));
                                    props.style = Some(grid_text_style_weekdays.clone());
                                    props.wrap = TextWrap::None;
                                    props.overflow = TextOverflow::Clip;
                                    props.color =
                                        Some(theme_weekdays.color_token("muted-foreground"));
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
                                        Some(theme_weekdays.color_token("muted-foreground"));
                                    props.layout = {
                                        let mut ls = LayoutStyle::default();
                                        ls.size.width = Length::Px(day_size);
                                        ls.size.height = Length::Auto;
                                        ls
                                    };
                                    cx.text_props(props)
                                }));
                                out
                            })
                            .layout(LayoutRefinement::default().w_px(MetricRef::Px(month_width)))
                            .into_element(cx);

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
                                    gap: day_col_gap.into(),
                                    padding: fret_core::Edges::all(Px(0.0)).into(),
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
                                let direction = crate::direction::use_direction(cx, None);
                                let month_model = month_model_days.clone();
                                let disabled_for_nav = Arc::clone(&disabled);
                                cx.roving_on_navigate(Arc::new(move |host, _cx, it| {
                                    use fret_core::KeyCode;
                                    use fret_ui::action::RovingNavigateResult;

                                    let Some(current) = it.current else {
                                        return RovingNavigateResult::NotHandled;
                                    };

                                    let step = calendar_day_grid_step_for_key(direction, it.key);

                                    if let Some(step) = step {
                                        let next = day_grid_step_target_skipping_disabled(
                                            current,
                                            it.len,
                                            step,
                                            disabled_for_nav.as_ref(),
                                        );
                                        return RovingNavigateResult::Handled {
                                            target: Some(next),
                                        };
                                    }

                                    if let Some(target) = calendar_day_grid_row_edge_target_for_key(
                                        direction, it.key, current, it.len,
                                    ) {
                                        let next = day_grid_row_edge_target_skipping_disabled(
                                            current,
                                            it.len,
                                            target,
                                            disabled_for_nav.as_ref(),
                                        );
                                        return RovingNavigateResult::Handled {
                                            target: Some(next),
                                        };
                                    }

                                    match it.key {
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
                                        let is_hidden = hidden.get(idx).copied().unwrap_or(true);
                                        if is_hidden {
                                            return calendar_range_hidden_day_cell(
                                                cx,
                                                &theme_days_for_days,
                                                day_size,
                                                week_row_gap,
                                            );
                                        }

                                        let is_today = today == day.date;
                                        let is_disabled =
                                            disabled.get(idx).copied().unwrap_or(false);

                                        let is_from = selected.from.is_some_and(|d| d == day.date);
                                        let is_to = selected.to.is_some_and(|d| d == day.date);
                                        let in_range = selected.contains(day.date);
                                        let selected_flag = in_range || is_from || is_to;
                                        let col = idx % 7;
                                        let is_selected_at = |idx: usize| {
                                            let Some(day) = grid.get(idx) else {
                                                return false;
                                            };
                                            let d = day.date;
                                            selected.contains(d)
                                                || selected.from.is_some_and(|it| it == d)
                                                || selected.to.is_some_and(|it| it == d)
                                        };
                                        let left_selected = col > 0 && is_selected_at(idx - 1);
                                        let right_selected = col < 6 && is_selected_at(idx + 1);

                                        calendar_range_day_cell(
                                            cx,
                                            &theme_days_for_days,
                                            locale,
                                            test_id_prefix.clone(),
                                            day.date,
                                            day.in_month,
                                            is_from,
                                            is_to,
                                            selected_flag,
                                            left_selected,
                                            right_selected,
                                            is_today,
                                            is_disabled,
                                            focus_date.is_some_and(|d| d == day.date),
                                            day_size,
                                            week_row_gap,
                                            &selected_model,
                                            required,
                                            min_days,
                                            max_days,
                                            exclude_disabled,
                                            close_on_select.clone(),
                                            modifiers.clone(),
                                            initial_focus_out.clone(),
                                            &day_button,
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
                                        gap: Px(0.0).into(),
                                        padding: fret_core::Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::Start,
                                        align: fret_ui::element::CrossAlign::Start,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        week_numbers
                                            .iter()
                                            .map(|n: &u32| {
                                                let mut props =
                                                    TextProps::new(Arc::from(n.to_string()));
                                                props.style =
                                                    Some(grid_text_style_week_numbers.clone());
                                                props.color = Some(
                                                    theme_days_for_week_numbers
                                                        .color_token("muted-foreground"),
                                                );
                                                props.wrap = TextWrap::None;
                                                props.overflow = TextOverflow::Clip;
                                                props.layout = {
                                                    let mut ls = LayoutStyle::default();
                                                    ls.size.width = Length::Px(day_size);
                                                    ls.size.height = Length::Px(day_size);
                                                    ls.margin.bottom =
                                                        fret_ui::element::MarginEdge::Px(
                                                            week_row_gap,
                                                        );
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
                                        gap: Px(0.0).into(),
                                        padding: fret_core::Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::Start,
                                        align: fret_ui::element::CrossAlign::Start,
                                        wrap: false,
                                    },
                                    move |_cx| vec![week_number_column, days_grid],
                                )
                            } else {
                                days_grid
                            };

                            let body = ui::v_stack(move |_cx| vec![weekday_row, days])
                                .gap(Space::N2)
                                .into_element(cx);

                            vec![header, body]
                        })
                        .gap(Space::N4)
                        .into_element(cx),
                    ]
                })]
            },
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn calendar_range_multi_month_view<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    is_row: bool,
    start_month: CalendarMonth,
    month_model: Model<CalendarMonth>,
    selected_model: Model<DateRangeSelection>,
    required: bool,
    min_days: i64,
    max_days: i64,
    exclude_disabled: bool,
    number_of_months: usize,
    locale: CalendarLocale,
    test_id_prefix: Option<Arc<str>>,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    disable_navigation: bool,
    week_start: Weekday,
    fixed_weeks: bool,
    weekday_labels: Arc<[Arc<str>]>,
    selected: DateRangeSelection,
    today: Date,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    day_size: Px,
    month_width: Px,
    day_grid_width: Px,
    week_row_gap: Px,
    modifiers: Arc<DayPickerModifiers>,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    grid_text_style: TextStyle,
    day_button: CalendarDayButton,
) -> Vec<AnyElement> {
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

    let nav = calendar_multi_month_nav_overlay(
        cx,
        CalendarMultiMonthNavProps {
            theme: theme.clone(),
            start_month,
            month_model: month_model.clone(),
            number_of_months,
            month_bounds,
            disable_navigation,
            day_size,
            months_span,
            test_id_prefix: None,
        },
    );

    let months_el = if is_row {
        ui::h_flex(move |cx| {
            months
                .iter()
                .copied()
                .map(|m| {
                    let month_el = calendar_range_month_view(
                        cx,
                        theme,
                        m,
                        locale,
                        test_id_prefix.clone(),
                        month_bounds,
                        week_start,
                        fixed_weeks,
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
                        required,
                        min_days,
                        max_days,
                        exclude_disabled,
                        modifiers.clone(),
                        close_on_select.clone(),
                        initial_focus_out.clone(),
                        grid_text_style.clone(),
                        day_button.clone(),
                    );

                    let month_test_id = test_id_prefix
                        .as_ref()
                        .map(|prefix| Arc::from(format!("{prefix}.month:{}", m.first_day())));

                    if let Some(month_test_id) = month_test_id {
                        month_el.test_id(month_test_id)
                    } else {
                        month_el
                    }
                })
                .collect::<Vec<_>>()
        })
        .gap(Space::N4)
        .items_start()
        .w_px(MetricRef::Px(months_span))
        .into_element(cx)
    } else {
        ui::v_flex(move |cx| {
            months
                .iter()
                .copied()
                .map(|m| {
                    let month_el = calendar_range_month_view(
                        cx,
                        theme,
                        m,
                        locale,
                        test_id_prefix.clone(),
                        month_bounds,
                        week_start,
                        fixed_weeks,
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
                        required,
                        min_days,
                        max_days,
                        exclude_disabled,
                        modifiers.clone(),
                        close_on_select.clone(),
                        initial_focus_out.clone(),
                        grid_text_style.clone(),
                        day_button.clone(),
                    );

                    let month_test_id = test_id_prefix
                        .as_ref()
                        .map(|prefix| Arc::from(format!("{prefix}.month:{}", m.first_day())));

                    if let Some(month_test_id) = month_test_id {
                        month_el.test_id(month_test_id)
                    } else {
                        month_el
                    }
                })
                .collect::<Vec<_>>()
        })
        .gap(Space::N4)
        .items_start()
        .w_px(MetricRef::Px(month_width))
        .into_element(cx)
    };

    let stack = ui::stack(move |_cx| vec![months_el, nav])
        .relative()
        .w_px(MetricRef::Px(months_span))
        .into_element(cx);

    vec![stack]
}

#[allow(clippy::too_many_arguments)]
fn calendar_range_month_view<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    month: CalendarMonth,
    locale: CalendarLocale,
    test_id_prefix: Option<Arc<str>>,
    month_bounds: Option<(CalendarMonth, CalendarMonth)>,
    week_start: Weekday,
    fixed_weeks: bool,
    weekday_labels: Arc<[Arc<str>]>,
    selected: DateRangeSelection,
    today: Date,
    show_outside_days: bool,
    disable_outside_days: bool,
    show_week_number: bool,
    day_size: Px,
    month_width: Px,
    day_grid_width: Px,
    week_row_gap: Px,
    month_model: Model<CalendarMonth>,
    selected_model: Model<DateRangeSelection>,
    required: bool,
    min_days: i64,
    max_days: i64,
    exclude_disabled: bool,
    modifiers: Arc<DayPickerModifiers>,
    close_on_select: Option<Model<bool>>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    grid_text_style: TextStyle,
    day_button: CalendarDayButton,
) -> AnyElement {
    let grid = if fixed_weeks {
        month_grid(month, week_start).to_vec()
    } else {
        month_grid_compact(month, week_start)
    };
    let in_bounds =
        |d: Date| month_bounds.map_or(true, |b| crate::calendar::date_in_month_bounds(d, b));

    let mut hidden = Vec::with_capacity(grid.len());
    let mut disabled = Vec::with_capacity(grid.len());
    for day in grid.iter() {
        let st = day_picker_cell_state(
            *day,
            show_outside_days,
            disable_outside_days,
            in_bounds(day.date),
            modifiers.as_ref(),
        );
        hidden.push(st.hidden);
        disabled.push(st.disabled);
    }
    let disabled: Arc<[bool]> = disabled.into();
    let hidden: Arc<[bool]> = hidden.into();

    let focus_date = {
        let preferred = selected.from.or(selected.to);
        let preferred_idx = preferred.and_then(|d| grid.iter().position(|it| it.date == d));
        let today_idx = grid.iter().position(|it| it.date == today);

        let visible =
            |idx: usize| !hidden.get(idx).copied().unwrap_or(true) && grid.get(idx).is_some();
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
                    .find(|(idx, _day)| visible(*idx) && enabled(*idx))
                    .and_then(|(idx, _day)| grid.get(idx).map(|d| d.date))
            })
    };

    let title = locale.month_title(month.month, month.year);

    let theme_days_for_days = theme.clone();
    let theme_days_for_week_numbers = theme.clone();

    let grid_text_style_caption = grid_text_style.clone();
    let grid_text_style_weekdays = grid_text_style.clone();
    let grid_text_style_week_numbers = grid_text_style.clone();

    let month_caption = calendar_month_caption(
        cx,
        title.clone(),
        grid_text_style_caption,
        day_size,
        month_width,
    );

    let weekday_row = calendar_weekday_row(
        cx,
        theme,
        weekday_labels.clone(),
        grid_text_style_weekdays,
        show_week_number,
        day_size,
        month_width,
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
            gap: Px(0.0).into(),
            padding: fret_core::Edges::all(Px(0.0)).into(),
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
        let direction = crate::direction::use_direction(cx, None);
        let month_model = month_model.clone();
        let disabled_for_nav = Arc::clone(&disabled);
        cx.roving_on_navigate(Arc::new(move |host, _cx, it| {
            use fret_core::KeyCode;
            use fret_ui::action::RovingNavigateResult;

            let Some(current) = it.current else {
                return RovingNavigateResult::NotHandled;
            };

            let step = calendar_day_grid_step_for_key(direction, it.key);

            if let Some(step) = step {
                let next = day_grid_step_target_skipping_disabled(
                    current,
                    it.len,
                    step,
                    disabled_for_nav.as_ref(),
                );
                return RovingNavigateResult::Handled { target: Some(next) };
            }

            if let Some(target) =
                calendar_day_grid_row_edge_target_for_key(direction, it.key, current, it.len)
            {
                let next = day_grid_row_edge_target_skipping_disabled(
                    current,
                    it.len,
                    target,
                    disabled_for_nav.as_ref(),
                );
                return RovingNavigateResult::Handled { target: Some(next) };
            }

            match it.key {
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
                let is_hidden = hidden.get(idx).copied().unwrap_or(true);
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
                let col = idx % 7;
                let is_selected_at = |idx: usize| {
                    let Some(day) = grid.get(idx) else {
                        return false;
                    };
                    let d = day.date;
                    selected.contains(d)
                        || selected.from.is_some_and(|it| it == d)
                        || selected.to.is_some_and(|it| it == d)
                };
                let left_selected = col > 0 && is_selected_at(idx - 1);
                let right_selected = col < 6 && is_selected_at(idx + 1);

                calendar_range_day_cell(
                    cx,
                    &theme_days_for_days,
                    locale,
                    test_id_prefix.clone(),
                    day.date,
                    day.in_month,
                    is_from,
                    is_to,
                    selected_flag,
                    left_selected,
                    right_selected,
                    is_today,
                    is_disabled,
                    focus_date.is_some_and(|d| d == day.date),
                    day_size,
                    week_row_gap,
                    &selected_model,
                    required,
                    min_days,
                    max_days,
                    exclude_disabled,
                    close_on_select.clone(),
                    modifiers.clone(),
                    initial_focus_out.clone(),
                    &day_button,
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
                gap: Px(0.0).into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
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
                gap: Px(0.0).into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: fret_ui::element::CrossAlign::Start,
                wrap: false,
            },
            move |_cx| vec![week_number_column, days_grid],
        )
    } else {
        days_grid
    };

    let body = ui::v_stack(move |_cx| vec![weekday_row, days])
        .gap(Space::N2)
        .into_element(cx);

    ui::v_stack(move |_cx| vec![month_caption, body])
        .gap(Space::N4)
        .into_element(cx)
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

fn calendar_range_hidden_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
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
        // Keep margins on the pressable node so row gaps don't inflate the chrome/background quad.
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

        let children = move |_cx: &mut ElementContext<'_, H>| Vec::<AnyElement>::new();
        (pressable, chrome_props, children)
    })
}

#[allow(clippy::too_many_arguments)]
fn calendar_range_day_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    locale: CalendarLocale,
    test_id_prefix: Option<Arc<str>>,
    date: Date,
    in_month: bool,
    is_from: bool,
    is_to: bool,
    selected: bool,
    left_selected: bool,
    right_selected: bool,
    today: bool,
    disabled: bool,
    focus_candidate: bool,
    size: Px,
    week_row_gap: Px,
    selected_model: &Model<DateRangeSelection>,
    required: bool,
    min_days: i64,
    max_days: i64,
    exclude_disabled: bool,
    close_on_select: Option<Model<bool>>,
    modifiers: Arc<DayPickerModifiers>,
    initial_focus_out: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    day_button: &CalendarDayButton,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);
    layout.margin.bottom = fret_ui::element::MarginEdge::Px(week_row_gap);

    let muted_fg = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let fg = if in_month {
        theme.color_token("foreground")
    } else {
        muted_fg
    };

    let (bg, fg) = if is_from || is_to {
        (
            theme.color_token("primary"),
            theme.color_token("primary-foreground"),
        )
    } else if selected || today {
        (
            theme.color_token("accent"),
            theme.color_token("accent-foreground"),
        )
    } else {
        (Color::TRANSPARENT, fg)
    };

    let day = date.day();
    let day_text: Arc<str> = Arc::from(day.to_string());
    let date_label = locale.day_aria_label(date, today, selected);
    let base_test_id = if let Some(prefix) = test_id_prefix {
        format!("{prefix}:{date}")
    } else {
        date.to_string()
    };
    let test_id: Arc<str> = if in_month {
        Arc::from(base_test_id)
    } else {
        Arc::from(format!("{base_test_id}:outside"))
    };

    let text_sm_px = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let text_sm_line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let supporting_text = calendar_day_button_supporting_text(
        day_button,
        CalendarDayButtonInfo {
            date,
            in_month,
            selected,
            today,
            disabled,
            range_start: is_from,
            range_end: is_to,
            range_middle: selected && !is_from && !is_to,
        },
    );
    let supporting_test_id = supporting_text
        .as_ref()
        .map(|_| Arc::from(format!("{test_id}:supporting-text")));
    let day_button = day_button.clone();

    control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
        if focus_candidate
            && !disabled
            && let Some(out) = initial_focus_out.as_ref()
            && out.get().is_none()
        {
            out.set(Some(id));
        }

        // (tests live at module scope)

        let selected_model = selected_model.clone();
        let close_on_select = close_on_select.clone();
        let modifiers = modifiers.clone();

        let complete_out = Rc::new(Cell::new(false));
        let complete_out_for_update = Rc::clone(&complete_out);

        cx.pressable_add_on_activate(Arc::new(move |host, _acx, _reason| {
            if disabled {
                return;
            }

            complete_out_for_update.set(false);
            let complete_out_in_update = Rc::clone(&complete_out_for_update);
            let _ = host.models_mut().update(&selected_model, |v| {
                let predicate = |d: Date| modifiers.disabled.iter().any(|m| m.is_match(d));
                let disabled_predicate =
                    (!modifiers.disabled.is_empty()).then_some(&predicate as &dyn Fn(Date) -> bool);
                v.apply_click_with(
                    date,
                    min_days,
                    max_days,
                    required,
                    exclude_disabled,
                    disabled_predicate,
                );
                complete_out_in_update.set(v.is_complete());
            });

            if complete_out_for_update.get()
                && let Some(open) = close_on_select.as_ref()
            {
                let _ = host.models_mut().update(open, |v| *v = false);
            }
        }));

        let accent_bg = theme.color_token("accent");
        let pressed_bg = {
            let mut c = accent_bg;
            c.a *= 0.85;
            c
        };

        let bg = if is_from || is_to {
            bg
        } else if selected || today {
            bg
        } else if st.pressed {
            pressed_bg
        } else if st.hovered {
            accent_bg
        } else {
            Color::TRANSPARENT
        };

        let mut chrome = day_button
            .chrome
            .clone()
            .merge(ChromeRefinement::default().bg(ColorRef::Color(bg)));
        if selected {
            // Match shadcn-web: range selection rounds only the visible row edges.
            let row_edge_radius = day_button
                .chrome
                .radius
                .clone()
                .unwrap_or_else(|| MetricRef::radius(Radius::Md));
            let mut radii = Corners4::all(MetricRef::Px(Px(0.0)));
            if !left_selected {
                radii.top_left = row_edge_radius.clone();
                radii.bottom_left = row_edge_radius.clone();
            }
            if !right_selected {
                radii.top_right = row_edge_radius.clone();
                radii.bottom_right = row_edge_radius.clone();
            }
            chrome = chrome.merge(ChromeRefinement::default().corner_radii(radii));
        }

        let mut chrome_props =
            decl_style::container_props(theme, chrome, day_button.layout.clone());
        // Keep margins on the pressable node so row gaps don't inflate the chrome/background quad.
        chrome_props.layout.margin = Default::default();

        let pressable = PressableProps {
            layout,
            enabled: !disabled,
            focusable: !disabled,
            focus_ring: Some(decl_style::focus_ring(
                theme,
                theme.metric_token("metric.radius.md"),
            )),
            a11y: PressableA11y {
                label: Some(date_label.clone()),
                test_id: Some(test_id.clone()),
                selected,
                ..Default::default()
            },
            ..Default::default()
        };

        let children = move |cx: &mut ElementContext<'_, H>| {
            calendar_day_button_children(
                cx,
                theme,
                day_text.clone(),
                supporting_text.clone(),
                supporting_test_id.clone(),
                text_sm_px,
                text_sm_line_height,
                muted_fg,
                fg,
                disabled,
                today,
                selected,
            )
        };

        (pressable, chrome_props, children)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};
    use time::Month;

    fn count_svg_icons(node: &AnyElement) -> usize {
        let mut out = usize::from(matches!(node.kind, ElementKind::SvgIcon(_)));
        for child in &node.children {
            out += count_svg_icons(child);
        }
        out
    }

    fn find_pressable_by_label<'a>(node: &'a AnyElement, label: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Pressable(props) = &node.kind
            && props.a11y.label.as_deref() == Some(label)
        {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = find_pressable_by_label(child, label) {
                return Some(found);
            }
        }

        None
    }

    fn find_element_by_test_id<'a>(node: &'a AnyElement, test_id: &str) -> Option<&'a AnyElement> {
        if node
            .semantics_decoration
            .as_ref()
            .and_then(|semantics| semantics.test_id.as_deref())
            == Some(test_id)
        {
            return Some(node);
        }

        if let ElementKind::Pressable(props) = &node.kind
            && props.a11y.test_id.as_deref() == Some(test_id)
        {
            return Some(node);
        }

        node.children
            .iter()
            .find_map(|child| find_element_by_test_id(child, test_id))
    }

    #[test]
    fn range_exclude_disabled_uses_modifiers_disabled_matchers() {
        let d1 = Date::from_calendar_date(2026, Month::January, 1).unwrap();
        let d2 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let d10 = Date::from_calendar_date(2026, Month::January, 10).unwrap();

        let mut modifiers = DayPickerModifiers::default();
        modifiers.disabled.push(DayMatcher::Date(d2));

        let predicate = |d: Date| modifiers.disabled.iter().any(|m| m.is_match(d));
        let disabled_predicate = Some(&predicate as &dyn Fn(Date) -> bool);

        let mut sel = DateRangeSelection::default();
        sel.apply_click_with(d1, 0, 0, false, false, None);
        sel.apply_click_with(d10, 0, 0, false, true, disabled_predicate);

        assert_eq!(
            sel,
            DateRangeSelection {
                from: Some(d10),
                to: None,
            }
        );
    }

    #[test]
    fn calendar_range_day_button_supporting_text_renders_only_for_in_month_days() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2026, Month::December));
            let selected = cx.app.models_mut().insert(DateRangeSelection {
                from: Some(Date::from_calendar_date(2026, Month::December, 8).unwrap()),
                to: Some(Date::from_calendar_date(2026, Month::December, 18).unwrap()),
            });

            let el = CalendarRange::new(month, selected)
                .test_id_prefix("calendar.range.test")
                .day_button(CalendarDayButton::new().supporting_text_by(|info| {
                    (info.in_month && info.range_middle).then(|| Arc::<str>::from("$100"))
                }))
                .into_element(cx);

            assert!(
                find_element_by_test_id(&el, "calendar.range.test:2026-12-12:supporting-text")
                    .is_some(),
                "expected in-month range-middle supporting text test id to render"
            );
            assert!(
                find_element_by_test_id(
                    &el,
                    "calendar.range.test:2026-11-29:outside:supporting-text"
                )
                .is_none(),
                "expected outside-day supporting text to remain absent"
            );
        });
    }

    #[test]
    fn calendar_range_dropdown_caption_layout_renders_month_and_year_triggers() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(280.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2026, Month::December));
            let selected = cx.app.models_mut().insert(DateRangeSelection {
                from: Some(Date::from_calendar_date(2026, Month::December, 8).unwrap()),
                to: Some(Date::from_calendar_date(2026, Month::December, 18).unwrap()),
            });

            let el = CalendarRange::new(month, selected)
                .test_id_prefix("calendar.range.caption")
                .caption_layout(CalendarCaptionLayout::Dropdown)
                .into_element(cx);

            assert!(
                find_element_by_test_id(&el, "calendar.range.caption.caption-month-trigger")
                    .is_some(),
                "expected range dropdown caption month trigger to render"
            );
            assert!(
                find_element_by_test_id(&el, "calendar.range.caption.caption-year-trigger")
                    .is_some(),
                "expected range dropdown caption year trigger to render"
            );
        });
    }

    #[test]
    fn calendar_range_nav_buttons_render_svg_icons() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(2026, Month::March));
            let selected = cx.app.models_mut().insert(DateRangeSelection::default());

            CalendarRange::new(month, selected).into_element(cx)
        });

        let prev = find_pressable_by_label(&element, "Go to the Previous Month")
            .expect("expected previous-month nav button");
        let next = find_pressable_by_label(&element, "Go to the Next Month")
            .expect("expected next-month nav button");

        assert_eq!(count_svg_icons(prev), 1);
        assert_eq!(count_svg_icons(next), 1);
    }
}
