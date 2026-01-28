use time::{Date, Duration, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolarHijriDate {
    pub year: i32,
    /// 1..=12
    pub month: u8,
    /// 1..=31
    pub day: u8,
}

impl SolarHijriDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolarHijriMonth {
    pub year: i32,
    /// 1..=12
    pub month: u8,
}

impl SolarHijriMonth {
    pub fn new(year: i32, month: u8) -> Self {
        Self { year, month }
    }

    pub fn from_gregorian(date: Date) -> Self {
        let d = solar_hijri_from_gregorian(date);
        Self::new(d.year, d.month)
    }

    pub fn first_day_gregorian(self) -> Date {
        solar_hijri_to_gregorian(SolarHijriDate::new(self.year, self.month, 1))
    }

    pub fn next_month(self) -> Self {
        if self.month >= 12 {
            Self::new(self.year + 1, 1)
        } else {
            Self::new(self.year, self.month + 1)
        }
    }

    pub fn prev_month(self) -> Self {
        if self.month <= 1 {
            Self::new(self.year - 1, 12)
        } else {
            Self::new(self.year, self.month - 1)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolarHijriCalendarDay {
    pub date: Date,
    pub solar: SolarHijriDate,
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

/// Builds a compact calendar grid for the given Solar Hijri month.
///
/// This mirrors `react-day-picker`'s default behavior:
/// - align the grid to week boundaries (start at `week_start`, end at the corresponding week end),
/// - variable number of rows (typically 5 or 6),
/// - includes outside-month days with `in_month=false`.
pub fn solar_hijri_month_grid_compact(
    month: SolarHijriMonth,
    week_start: Weekday,
) -> Vec<SolarHijriCalendarDay> {
    let first = month.first_day_gregorian();
    let next_first = month.next_month().first_day_gregorian();
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
            let solar = solar_hijri_from_gregorian(date);
            SolarHijriCalendarDay {
                date,
                solar,
                in_month: solar.year == month.year && solar.month == month.month,
            }
        })
        .collect()
}

/// Converts a Gregorian (time::Date) date to a Solar Hijri (Jalaali) date.
pub fn solar_hijri_from_gregorian(date: Date) -> SolarHijriDate {
    let g = d2j(g2d(date.year(), date.month() as i32, date.day() as i32));
    SolarHijriDate {
        year: g.jy,
        month: g.jm as u8,
        day: g.jd as u8,
    }
}

/// Converts a Solar Hijri (Jalaali) date to a Gregorian (time::Date) date.
pub fn solar_hijri_to_gregorian(date: SolarHijriDate) -> Date {
    let g = d2g(j2d(date.year, date.month as i32, date.day as i32));
    Date::from_calendar_date(
        g.gy,
        time::Month::try_from(g.gm as u8).expect("valid Gregorian month"),
        g.gd as u8,
    )
    .expect("valid Gregorian date")
}

// -----------------------------------------------------------------------------
// Jalaali conversion algorithm (ported from jalaali-js).
// Reference: https://github.com/jalaali/jalaali-js
// -----------------------------------------------------------------------------

const BREAKS: [i32; 20] = [
    -61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262, 2324,
    2394, 2456, 3178,
];

#[derive(Debug, Clone, Copy)]
struct JalCal {
    gy: i32,
    march: i32,
    leap: i32,
}

#[derive(Debug, Clone, Copy)]
struct Jalaali {
    jy: i32,
    jm: i32,
    jd: i32,
}

#[derive(Debug, Clone, Copy)]
struct Gregorian {
    gy: i32,
    gm: i32,
    gd: i32,
}

fn div(a: i32, b: i32) -> i32 {
    // Match jalaali-js `div()` (trunc toward zero), not Euclidean division.
    a / b
}

fn modulo(a: i32, b: i32) -> i32 {
    // Match jalaali-js `mod()` (remainder with trunc toward zero), not Euclidean remainder.
    a - (a / b) * b
}

fn jal_cal(jy: i32, without_leap: bool) -> JalCal {
    let bl = BREAKS.len() as i32;
    let gy = jy + 621;
    let mut leap_j: i32 = -14;
    let mut jp = BREAKS[0];
    let mut jm = BREAKS[1];
    let mut jump = jm - jp;

    if jy < jp || jy >= BREAKS[(bl - 1) as usize] {
        panic!("invalid Solar Hijri year {jy}");
    }

    for i in 1..(bl as usize) {
        jm = BREAKS[i];
        jump = jm - jp;
        if jy < jm {
            break;
        }
        leap_j += div(jump, 33) * 8 + div(modulo(jump, 33), 4);
        jp = jm;
    }
    let mut n = jy - jp;

    leap_j += div(n, 33) * 8 + div(modulo(n, 33) + 3, 4);
    if modulo(jump, 33) == 4 && jump - n == 4 {
        leap_j += 1;
    }

    let leap_g = div(gy, 4) - div((div(gy, 100) + 1) * 3, 4) - 150;
    let march = 20 + leap_j - leap_g;

    if without_leap {
        return JalCal { gy, march, leap: 0 };
    }

    if jump - n < 6 {
        n = n - jump + div(jump + 4, 33) * 33;
    }
    let mut leap = modulo(modulo(n + 1, 33) - 1, 4);
    if leap == -1 {
        leap = 4;
    }

    JalCal { gy, march, leap }
}

fn j2d(jy: i32, jm: i32, jd: i32) -> i32 {
    let r = jal_cal(jy, true);
    g2d(r.gy, 3, r.march) + (jm - 1) * 31 - div(jm, 7) * (jm - 7) + jd - 1
}

fn d2j(jdn: i32) -> Jalaali {
    let g = d2g(jdn);
    let mut jy = g.gy - 621;
    let r = jal_cal(jy, false);
    let jdn1f = g2d(g.gy, 3, r.march);

    let mut k = jdn - jdn1f;
    if k >= 0 {
        if k <= 185 {
            let jm = 1 + div(k, 31);
            let jd = modulo(k, 31) + 1;
            return Jalaali { jy, jm, jd };
        }
        k -= 186;
    } else {
        jy -= 1;
        k += 179;
        if r.leap == 1 {
            k += 1;
        }
    }

    let jm = 7 + div(k, 30);
    let jd = modulo(k, 30) + 1;
    Jalaali { jy, jm, jd }
}

fn g2d(gy: i32, gm: i32, gd: i32) -> i32 {
    let d =
        div((gy + div(gm - 8, 6) + 100100) * 1461, 4) + div(153 * modulo(gm + 9, 12) + 2, 5) + gd
            - 34840408;
    d - div(div(gy + 100100 + div(gm - 8, 6), 100) * 3, 4) + 752
}

fn d2g(jdn: i32) -> Gregorian {
    let mut j = 4 * jdn + 139361631;
    j = j + div(div(4 * jdn + 183187720, 146097) * 3, 4) * 4 - 3908;
    let i = div(modulo(j, 1461), 4) * 5 + 308;
    let gd = div(modulo(i, 153), 5) + 1;
    let gm = modulo(div(i, 153), 12) + 1;
    let gy = div(j, 1461) - 100100 + div(8 - gm, 6);
    Gregorian { gy, gm, gd }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn solar_hijri_roundtrip_matches_known_reference() {
        let g = Date::from_calendar_date(2025, Month::June, 12).expect("valid date");
        let j = solar_hijri_from_gregorian(g);
        assert_eq!(j.year, 1404);
        assert_eq!(j.month, 3);
        assert_eq!(j.day, 22);

        let g2 = solar_hijri_to_gregorian(j);
        assert_eq!(g2, g);
    }
}
