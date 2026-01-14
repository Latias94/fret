use time::{Date, Duration, Month, Weekday};

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
}
