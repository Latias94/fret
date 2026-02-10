use std::sync::Arc;

use time::{Date, Duration, Month, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub start: Date,
    pub end: Date,
}

impl DateRange {
    pub fn new(a: Date, b: Date) -> Self {
        if a <= b {
            Self { start: a, end: b }
        } else {
            Self { start: b, end: a }
        }
    }

    pub fn contains(&self, date: Date) -> bool {
        self.start <= date && date <= self.end
    }
}

/// A DayPicker-like date range selection state (supports partial selection).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DateRangeSelection {
    pub from: Option<Date>,
    pub to: Option<Date>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionUpdate<T> {
    NoChange,
    Set(T),
}

impl<T> SelectionUpdate<T> {
    pub fn is_change(&self) -> bool {
        matches!(self, Self::Set(_))
    }
}

/// A subset of `react-day-picker`'s `Matcher` union, expressed in `time::Date`.
///
/// Upstream reference: `react-day-picker/src/types/shared.ts` (`export type Matcher = ...`).
#[derive(Clone)]
pub enum DayMatcher {
    Bool(bool),
    Predicate(Arc<dyn Fn(Date) -> bool + Send + Sync + 'static>),
    Date(Date),
    Dates(Arc<[Date]>),
    DateRange(DateRangeSelection),
    DateBefore { before: Date },
    DateAfter { after: Date },
    DateInterval { before: Date, after: Date },
    DayOfWeek(Arc<[Weekday]>),
}

impl std::fmt::Debug for DayMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => f.debug_tuple("Bool").field(v).finish(),
            Self::Predicate(_) => f.debug_tuple("Predicate").field(&"<fn>").finish(),
            Self::Date(d) => f.debug_tuple("Date").field(d).finish(),
            Self::Dates(ds) => f.debug_tuple("Dates").field(ds).finish(),
            Self::DateRange(r) => f.debug_tuple("DateRange").field(r).finish(),
            Self::DateBefore { before } => f
                .debug_struct("DateBefore")
                .field("before", before)
                .finish(),
            Self::DateAfter { after } => f.debug_struct("DateAfter").field("after", after).finish(),
            Self::DateInterval { before, after } => f
                .debug_struct("DateInterval")
                .field("before", before)
                .field("after", after)
                .finish(),
            Self::DayOfWeek(days) => f.debug_tuple("DayOfWeek").field(days).finish(),
        }
    }
}

impl DayMatcher {
    pub fn day_of_week(day: Weekday) -> Self {
        Self::DayOfWeek(Arc::from([day]))
    }

    pub fn day_of_week_any(days: impl Into<Arc<[Weekday]>>) -> Self {
        Self::DayOfWeek(days.into())
    }

    pub fn dates(dates: impl Into<Arc<[Date]>>) -> Self {
        Self::Dates(dates.into())
    }

    pub fn is_match(&self, date: Date) -> bool {
        // Mirrors `react-day-picker` `dateMatchModifiers()` semantics.
        match self {
            Self::Bool(v) => *v,
            Self::Predicate(p) => p(date),
            Self::Date(d) => *d == date,
            Self::Dates(ds) => ds.iter().any(|d| *d == date),
            Self::DateRange(r) => range_includes_date(*r, date, false),
            Self::DayOfWeek(days) => days.iter().any(|d| *d == date.weekday()),
            Self::DateInterval { before, after } => {
                let diff_before = (*before - date).whole_days();
                let diff_after = (*after - date).whole_days();
                let is_day_before = diff_before > 0;
                let is_day_after = diff_after < 0;
                let is_closed_interval = *before > *after;
                if is_closed_interval {
                    is_day_after && is_day_before
                } else {
                    is_day_before || is_day_after
                }
            }
            Self::DateAfter { after } => (date - *after).whole_days() > 0,
            Self::DateBefore { before } => (*before - date).whole_days() > 0,
        }
    }
}

/// Mirrors `react-day-picker` `rangeIncludesDate()` behavior (inclusive ends by default).
pub fn range_includes_date(range: DateRangeSelection, date: Date, exclude_ends: bool) -> bool {
    let mut from = range.from;
    let mut to = range.to;
    if let (Some(f), Some(t)) = (from, to) {
        if t < f {
            (from, to) = (Some(t), Some(f));
        }
    }

    match (from, to) {
        (Some(f), Some(t)) => {
            let left = (date - f).whole_days();
            let right = (t - date).whole_days();
            let min = if exclude_ends { 1 } else { 0 };
            left >= min && right >= min
        }
        (None, Some(t)) if !exclude_ends => t == date,
        (Some(f), None) if !exclude_ends => f == date,
        _ => false,
    }
}

/// A headless representation of `react-day-picker`'s "modifiers" input, focusing on the
/// built-in `disabled` and `hidden` buckets.
#[derive(Debug, Default, Clone)]
pub struct DayPickerModifiers {
    pub disabled: Vec<DayMatcher>,
    pub hidden: Vec<DayMatcher>,
}

impl DayPickerModifiers {
    pub fn disabled_by(mut self, matcher: DayMatcher) -> Self {
        self.disabled.push(matcher);
        self
    }

    pub fn hidden_by(mut self, matcher: DayMatcher) -> Self {
        self.hidden.push(matcher);
        self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DayPickerDayModifiers {
    pub outside: bool,
    pub disabled: bool,
    pub hidden: bool,
}

pub fn day_picker_day_modifiers(
    day: CalendarDay,
    show_outside_days: bool,
    modifiers: &DayPickerModifiers,
) -> DayPickerDayModifiers {
    let outside = !day.in_month;
    let hidden =
        (!show_outside_days && outside) || modifiers.hidden.iter().any(|m| m.is_match(day.date));
    let disabled = modifiers.disabled.iter().any(|m| m.is_match(day.date));
    DayPickerDayModifiers {
        outside,
        disabled,
        hidden,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPickerGridLayout {
    Compact,
    FixedWeeks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayPickerGridOptions {
    pub week_start: Weekday,
    pub layout: DayPickerGridLayout,
}

impl Default for DayPickerGridOptions {
    fn default() -> Self {
        Self {
            week_start: Weekday::Monday,
            layout: DayPickerGridLayout::Compact,
        }
    }
}

/// Build the month grid using `react-day-picker`-like options.
///
/// - `Compact`: variable number of weeks (default for shadcn's Calendar).
/// - `FixedWeeks`: always 6 weeks (42 days), useful for stable layouts.
pub fn day_picker_month_grid(
    month: CalendarMonth,
    options: DayPickerGridOptions,
) -> Vec<CalendarDay> {
    match options.layout {
        DayPickerGridLayout::Compact => month_grid_compact(month, options.week_start),
        DayPickerGridLayout::FixedWeeks => month_grid(month, options.week_start).to_vec(),
    }
}

/// Mirrors `react-day-picker` `useSingle` selection behavior.
pub fn day_picker_select_single(
    trigger: Date,
    current: Option<Date>,
    required: bool,
) -> SelectionUpdate<Option<Date>> {
    let mut next = Some(trigger);
    if !required && current.is_some_and(|d| d == trigger) {
        next = None;
    }

    if next == current {
        SelectionUpdate::NoChange
    } else {
        SelectionUpdate::Set(next)
    }
}

/// Mirrors `react-day-picker` `useMulti` selection behavior.
pub fn day_picker_select_multi(
    trigger: Date,
    current: &[Date],
    required: bool,
    min: Option<usize>,
    max: Option<usize>,
) -> SelectionUpdate<Vec<Date>> {
    let min = min.unwrap_or(0);
    let max = max.unwrap_or(0);

    let is_selected = current.iter().any(|d| *d == trigger);

    if is_selected {
        if current.len() == min {
            return SelectionUpdate::NoChange;
        }
        if required && current.len() == 1 {
            return SelectionUpdate::NoChange;
        }
        let next = current
            .iter()
            .copied()
            .filter(|d| *d != trigger)
            .collect::<Vec<_>>();
        if next == current {
            SelectionUpdate::NoChange
        } else {
            SelectionUpdate::Set(next)
        }
    } else {
        let next = if max > 0 && current.len() == max {
            vec![trigger]
        } else {
            let mut next = current.to_vec();
            next.push(trigger);
            next
        };
        if next == current {
            SelectionUpdate::NoChange
        } else {
            SelectionUpdate::Set(next)
        }
    }
}

/// Mirrors `react-day-picker` `addToRange()` selection behavior (including the
/// min/max "reset to start" logic).
///
/// Notes:
/// - `min_days`/`max_days` are "calendar-day differences" between `from` and
///   `to`, matching `date-fns` `differenceInCalendarDays` used upstream.
/// - When `exclude_disabled` is `true`, the returned range will reset to
///   `{ from: trigger, to: None }` if any day in the candidate range matches
///   `disabled_predicate`.
pub fn day_picker_add_to_range(
    trigger: Date,
    current: DateRangeSelection,
    min_days: i64,
    max_days: i64,
    required: bool,
    exclude_disabled: bool,
    disabled_predicate: Option<&dyn Fn(Date) -> bool>,
) -> SelectionUpdate<DateRangeSelection> {
    let mut from = current.from;
    let mut to = current.to;

    // If the state is somehow "to without from", treat it as empty.
    if from.is_none() && to.is_some() {
        from = None;
        to = None;
    }

    let mut next: Option<DateRangeSelection> = match (from, to) {
        (None, None) => Some(DateRangeSelection {
            from: Some(trigger),
            to: if min_days > 0 { None } else { Some(trigger) },
        }),
        (Some(f), None) => {
            if f == trigger {
                if min_days == 0 {
                    Some(DateRangeSelection {
                        from: Some(f),
                        to: Some(trigger),
                    })
                } else if required {
                    Some(DateRangeSelection {
                        from: Some(f),
                        to: None,
                    })
                } else {
                    None
                }
            } else if trigger < f {
                Some(DateRangeSelection {
                    from: Some(trigger),
                    to: Some(f),
                })
            } else {
                Some(DateRangeSelection {
                    from: Some(f),
                    to: Some(trigger),
                })
            }
        }
        (Some(f), Some(t)) => {
            if f == trigger && t == trigger {
                if required {
                    Some(DateRangeSelection {
                        from: Some(f),
                        to: Some(t),
                    })
                } else {
                    None
                }
            } else if f == trigger {
                Some(DateRangeSelection {
                    from: Some(f),
                    to: if min_days > 0 { None } else { Some(trigger) },
                })
            } else if t == trigger {
                Some(DateRangeSelection {
                    from: Some(trigger),
                    to: if min_days > 0 { None } else { Some(trigger) },
                })
            } else if trigger < f {
                Some(DateRangeSelection {
                    from: Some(trigger),
                    to: Some(t),
                })
            } else if trigger > f {
                Some(DateRangeSelection {
                    from: Some(f),
                    to: Some(trigger),
                })
            } else {
                // Mirrors upstream "Invalid range" branch: keep the current value.
                Some(DateRangeSelection {
                    from: Some(f),
                    to: Some(t),
                })
            }
        }
        (None, Some(_)) => Some(DateRangeSelection {
            from: Some(trigger),
            to: if min_days > 0 { None } else { Some(trigger) },
        }),
    };

    // Apply min/max constraints (upstream behavior: reset to the start of the range).
    if let Some(r) = next.as_mut()
        && let (Some(f), Some(t)) = (r.from, r.to)
    {
        let diff_days = (t - f).whole_days();
        if max_days > 0 && diff_days > max_days {
            *r = DateRangeSelection {
                from: Some(trigger),
                to: None,
            };
        } else if min_days > 1 && diff_days < min_days {
            *r = DateRangeSelection {
                from: Some(trigger),
                to: None,
            };
        }
    }

    // Apply exclude-disabled behavior (upstream behavior: reset to the start of the range).
    if exclude_disabled
        && let Some(pred) = disabled_predicate
        && let Some(r) = next.as_mut()
        && let (Some(f), Some(t)) = (r.from, r.to)
    {
        let diff_days = (t - f).whole_days();
        if diff_days >= 0 {
            for i in 0..=diff_days {
                let d = f + Duration::days(i);
                if pred(d) {
                    *r = DateRangeSelection {
                        from: Some(trigger),
                        to: None,
                    };
                    break;
                }
            }
        }
    }

    let next = next.unwrap_or_default();
    if next == current {
        SelectionUpdate::NoChange
    } else {
        SelectionUpdate::Set(next)
    }
}

impl DateRangeSelection {
    pub fn clear(&mut self) {
        self.from = None;
        self.to = None;
    }

    pub fn is_complete(&self) -> bool {
        self.from.is_some() && self.to.is_some()
    }

    pub fn normalized_range(&self) -> Option<DateRange> {
        Some(DateRange::new(self.from?, self.to?))
    }

    pub fn is_start(&self, date: Date) -> bool {
        self.from.is_some_and(|d| d == date)
            || self.to.is_some_and(|d| d == date && self.from.is_none())
    }

    pub fn is_end(&self, date: Date) -> bool {
        self.to.is_some_and(|d| d == date) && self.from.is_some()
    }

    pub fn contains(&self, date: Date) -> bool {
        self.normalized_range()
            .is_some_and(|range| range.contains(date))
    }

    /// Applies a DayPicker-like click interaction (mirrors upstream
    /// `react-day-picker` `addToRange()` defaults).
    ///
    /// This uses default selection options:
    /// - `min_days = 0`
    /// - `max_days = 0`
    /// - `required = false`
    /// - `exclude_disabled = false`
    pub fn apply_click(&mut self, date: Date) {
        let current = *self;
        if let SelectionUpdate::Set(next) =
            day_picker_add_to_range(date, current, 0, 0, false, false, None)
        {
            *self = next;
        }
    }

    pub fn apply_click_with(
        &mut self,
        date: Date,
        min_days: i64,
        max_days: i64,
        required: bool,
        exclude_disabled: bool,
        disabled_predicate: Option<&dyn Fn(Date) -> bool>,
    ) {
        let current = *self;
        if let SelectionUpdate::Set(next) = day_picker_add_to_range(
            date,
            current,
            min_days,
            max_days,
            required,
            exclude_disabled,
            disabled_predicate,
        ) {
            *self = next;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarMonth {
    pub year: i32,
    pub month: Month,
}

impl CalendarMonth {
    pub fn new(year: i32, month: Month) -> Self {
        Self { year, month }
    }

    pub fn from_date(date: Date) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    pub fn first_day(&self) -> Date {
        Date::from_calendar_date(self.year, self.month, 1).expect("valid month")
    }

    pub fn next_month(&self) -> Self {
        let (year, month) = if self.month == Month::December {
            (self.year + 1, Month::January)
        } else {
            (self.year, self.month.next())
        };
        Self { year, month }
    }

    pub fn prev_month(&self) -> Self {
        let (year, month) = if self.month == Month::January {
            (self.year - 1, Month::December)
        } else {
            (self.year, self.month.previous())
        };
        Self { year, month }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDay {
    pub date: Date,
    pub in_month: bool,
}

fn weekday_index_from_monday(weekday: Weekday) -> u8 {
    weekday.number_from_monday()
}

fn offset_to_week_start(day: Weekday, week_start: Weekday) -> u8 {
    let a = weekday_index_from_monday(day) as i16;
    let b = weekday_index_from_monday(week_start) as i16;
    let diff = a - b;
    ((diff % 7 + 7) % 7) as u8
}

/// Builds a 6-week (42-day) calendar grid for the given month.
///
/// This matches common date picker UIs:
/// - always returns 42 days (stable layout)
/// - includes outside-month days with `in_month=false`
pub fn month_grid(month: CalendarMonth, week_start: Weekday) -> [CalendarDay; 42] {
    let first = month.first_day();
    let start_offset = offset_to_week_start(first.weekday(), week_start) as i64;
    let grid_start = first - Duration::days(start_offset);

    std::array::from_fn(|i| {
        let date = grid_start + Duration::days(i as i64);
        CalendarDay {
            date,
            in_month: date.year() == month.year && date.month() == month.month,
        }
    })
}

/// Builds a compact calendar grid for the given month.
///
/// This matches `react-day-picker`'s default behavior used by shadcn's `Calendar`:
/// - the grid is aligned to week boundaries (start at `week_start`, end at the corresponding week end)
/// - the number of rows is variable (typically 5 or 6; 4 is possible for some Februaries)
/// - outside-month days are included with `in_month=false`
pub fn month_grid_compact(month: CalendarMonth, week_start: Weekday) -> Vec<CalendarDay> {
    let first = month.first_day();
    let next_first = month.next_month().first_day();
    let last = next_first - Duration::days(1);

    let start_offset = offset_to_week_start(first.weekday(), week_start) as i64;
    let grid_start = first - Duration::days(start_offset);

    let week_start_idx = weekday_index_from_monday(week_start) as i16;
    let week_end_idx = (week_start_idx + 6) % 7;
    let last_idx = weekday_index_from_monday(last.weekday()) as i16;
    let end_offset = ((week_end_idx - last_idx) % 7 + 7) % 7;
    let grid_end = last + Duration::days(end_offset as i64);

    let days = (grid_end - grid_start).whole_days() + 1;
    debug_assert!(days > 0 && days % 7 == 0);

    (0..days)
        .map(|i| {
            let date = grid_start + Duration::days(i);
            CalendarDay {
                date,
                in_month: date.year() == month.year && date.month() == month.month,
            }
        })
        .collect()
}

fn start_of_week(date: Date, week_start: Weekday) -> Date {
    let offset = offset_to_week_start(date.weekday(), week_start) as i64;
    date - Duration::days(offset)
}

/// Returns a week number aligned to `week_start`, matching `date-fns` `getWeek` defaults
/// (`firstWeekContainsDate = 1`).
///
/// This is the numbering used by `react-day-picker` when `showWeekNumber` is enabled.
pub fn week_number(date: Date, week_start: Weekday) -> u32 {
    let week_start_date = start_of_week(date, week_start);
    let week_end_date = week_start_date + Duration::days(6);
    let week_year = week_end_date.year();

    let jan1 = Date::from_calendar_date(week_year, Month::January, 1).expect("valid year");
    let week1_start = start_of_week(jan1, week_start);

    let diff_days = (week_start_date - week1_start).whole_days();
    let weeks = diff_days.div_euclid(7).max(0);
    (weeks as u32) + 1
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_grid_is_stable_42_days() {
        let m = CalendarMonth::new(2026, Month::January);
        let grid = month_grid(m, Weekday::Monday);
        assert_eq!(grid.len(), 42);
    }

    #[test]
    fn month_grid_includes_first_day_and_marks_in_month() {
        let m = CalendarMonth::new(2026, Month::January);
        let grid = month_grid(m, Weekday::Monday);
        let jan1 = Date::from_calendar_date(2026, Month::January, 1).unwrap();
        assert!(grid.iter().any(|d| d.date == jan1 && d.in_month));
    }

    #[test]
    fn month_nav_rolls_year_boundaries() {
        let dec = CalendarMonth::new(2025, Month::December);
        assert_eq!(dec.next_month(), CalendarMonth::new(2026, Month::January));

        let jan = CalendarMonth::new(2026, Month::January);
        assert_eq!(jan.prev_month(), CalendarMonth::new(2025, Month::December));
    }

    #[test]
    fn date_range_new_orders_start_end() {
        let a = Date::from_calendar_date(2026, Month::January, 5).unwrap();
        let b = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let range = DateRange::new(a, b);
        assert_eq!(range.start, b);
        assert_eq!(range.end, a);
        assert!(range.contains(a));
        assert!(range.contains(b));
    }

    #[test]
    fn date_range_selection_click_sequence_matches_daypicker_expectations() {
        let d1 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let d2 = Date::from_calendar_date(2026, Month::January, 5).unwrap();
        let d3 = Date::from_calendar_date(2026, Month::January, 8).unwrap();

        let mut sel = DateRangeSelection::default();
        sel.apply_click(d2);
        assert_eq!(sel.from, Some(d2));
        assert_eq!(sel.to, Some(d2));
        assert!(sel.is_complete());

        sel.apply_click(d1);
        assert_eq!(sel.from, Some(d1));
        assert_eq!(sel.to, Some(d2));
        assert!(sel.is_complete());
        assert!(sel.contains(d1));
        assert!(sel.contains(d2));

        sel.apply_click(d3);
        assert_eq!(sel.from, Some(d1));
        assert_eq!(sel.to, Some(d3));
        assert!(sel.is_complete());
    }

    #[test]
    fn day_picker_single_optional_toggles_off_on_same_day() {
        let d1 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        assert_eq!(
            day_picker_select_single(d1, Some(d1), false),
            SelectionUpdate::Set(None)
        );
        assert_eq!(
            day_picker_select_single(d1, Some(d1), true),
            SelectionUpdate::NoChange
        );
    }

    #[test]
    fn day_picker_multi_resets_when_max_reached() {
        let d1 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let d2 = Date::from_calendar_date(2026, Month::January, 3).unwrap();
        let d3 = Date::from_calendar_date(2026, Month::January, 4).unwrap();

        let cur = vec![d1, d2];
        assert_eq!(
            day_picker_select_multi(d3, &cur, false, None, Some(2)),
            SelectionUpdate::Set(vec![d3])
        );
    }

    #[test]
    fn day_picker_range_min_and_exclude_disabled_match_upstream_intent() {
        let d1 = Date::from_calendar_date(2026, Month::January, 1).unwrap();
        let d2 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let d10 = Date::from_calendar_date(2026, Month::January, 10).unwrap();

        // min_days > 0 => first click produces partial selection (to=None).
        let mut sel = DateRangeSelection::default();
        sel.apply_click_with(d1, 1, 0, false, false, None);
        assert_eq!(
            sel,
            DateRangeSelection {
                from: Some(d1),
                to: None
            }
        );

        // exclude_disabled => selecting a range that spans a disabled day resets to start.
        let disabled = |d: Date| d == d2;
        let mut sel = DateRangeSelection::default();
        sel.apply_click_with(d1, 0, 0, false, false, None);
        sel.apply_click_with(d10, 0, 0, false, true, Some(&disabled));
        assert_eq!(
            sel,
            DateRangeSelection {
                from: Some(d10),
                to: None
            }
        );
    }

    #[test]
    fn range_includes_date_matches_endpoints_only_when_open_ended() {
        let d1 = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let d2 = Date::from_calendar_date(2026, Month::January, 3).unwrap();
        let only_from = DateRangeSelection {
            from: Some(d1),
            to: None,
        };
        let only_to = DateRangeSelection {
            from: None,
            to: Some(d1),
        };

        assert!(range_includes_date(only_from, d1, false));
        assert!(!range_includes_date(only_from, d2, false));
        assert!(range_includes_date(only_to, d1, false));
        assert!(!range_includes_date(only_to, d2, false));
    }

    #[test]
    fn day_matcher_date_interval_excludes_ends() {
        let after = Date::from_calendar_date(2026, Month::January, 2).unwrap();
        let before = Date::from_calendar_date(2026, Month::January, 5).unwrap();
        let mid = Date::from_calendar_date(2026, Month::January, 3).unwrap();
        let m = DayMatcher::DateInterval { before, after };

        assert!(!m.is_match(after));
        assert!(m.is_match(mid));
        assert!(!m.is_match(before));
    }

    #[test]
    fn day_matcher_day_of_week_matches_any() {
        let monday = Date::from_calendar_date(2026, Month::January, 5).unwrap();
        assert_eq!(monday.weekday(), Weekday::Monday);
        let m = DayMatcher::day_of_week_any(Arc::from([Weekday::Sunday, Weekday::Monday]));
        assert!(m.is_match(monday));
    }

    #[test]
    fn day_picker_month_grid_fixed_weeks_is_42_days() {
        let m = CalendarMonth::new(2026, Month::January);
        let grid = day_picker_month_grid(
            m,
            DayPickerGridOptions {
                week_start: Weekday::Monday,
                layout: DayPickerGridLayout::FixedWeeks,
            },
        );
        assert_eq!(grid.len(), 42);
    }
}
