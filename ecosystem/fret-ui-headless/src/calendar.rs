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

    /// Applies a click interaction:
    /// - first click sets `from`,
    /// - second click sets `to` (swapping if needed),
    /// - third click starts a new range (resets to the clicked date).
    pub fn apply_click(&mut self, date: Date) {
        match (self.from, self.to) {
            (None, None) => {
                self.from = Some(date);
            }
            (Some(from), None) => {
                self.from = Some(from);
                self.to = Some(date);
                if let Some(range) = self.normalized_range() {
                    self.from = Some(range.start);
                    self.to = Some(range.end);
                }
            }
            (Some(_), Some(_)) => {
                self.from = Some(date);
                self.to = None;
            }
            (None, Some(_)) => {
                self.from = Some(date);
                self.to = None;
            }
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
        assert_eq!(sel.to, None);
        assert!(!sel.is_complete());

        sel.apply_click(d1);
        assert_eq!(sel.from, Some(d1));
        assert_eq!(sel.to, Some(d2));
        assert!(sel.is_complete());
        assert!(sel.contains(d1));
        assert!(sel.contains(d2));

        sel.apply_click(d3);
        assert_eq!(sel.from, Some(d3));
        assert_eq!(sel.to, None);
        assert!(!sel.is_complete());
    }
}
