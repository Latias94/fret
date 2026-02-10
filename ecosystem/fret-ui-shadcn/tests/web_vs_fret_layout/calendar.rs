use super::*;

#[test]
fn web_vs_fret_layout_calendar_demo_day_grid_geometry_and_a11y_labels_match_web() {
    let web = read_web_golden("calendar-demo");
    let theme = web_theme(&web);

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Sunday, December 28th, 2025")
    })
    .expect("web day button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::{Month, Weekday};

        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

        vec![
            fret_ui_shadcn::Calendar::new(month, selected)
                .week_start(Weekday::Sunday)
                .disable_outside_days(false)
                .into_element(cx),
        ]
    });

    fn is_calendar_day_label(label: &str) -> bool {
        // Examples:
        // - "Sunday, December 28th, 2025"
        // - "Thursday, June 12th, 2025, selected"
        let label = label.strip_suffix(", selected").unwrap_or(label);
        let label = label.strip_prefix("Today, ").unwrap_or(label);
        if !label.contains(',') {
            return false;
        }
        let Some((_weekday, rest)) = label.split_once(", ") else {
            return false;
        };
        let Some((_month_and_day, year)) = rest.rsplit_once(", ") else {
            return false;
        };
        if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        label.contains("st, ")
            || label.contains("nd, ")
            || label.contains("rd, ")
            || label.contains("th, ")
    }

    let day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| is_calendar_day_label(label))
        })
        .count();
    assert_eq!(
        day_buttons, 35,
        "expected a 5-week (35-day) grid for January 2026 when week starts on Sunday"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        "calendar prev button width",
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar prev button height",
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Sunday, December 28th, 2025"),
    )
    .expect("fret day semantics node");
    assert_close_px(
        "calendar day button width",
        day.bounds.size.width,
        web_day.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar day button height",
        day.bounds.size.height,
        web_day.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px("calendar day x", node_bounds.origin.x, web_day.rect.x, 3.0);
    assert_close_px("calendar day y", node_bounds.origin.y, web_day.rect.y, 3.0);
}

#[test]
fn web_vs_fret_layout_calendar_hijri_day_grid_geometry_and_a11y_labels_match_web() {
    let web = read_web_golden("calendar-hijri");
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;
    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_month_grid =
        web_find_by_class_token(&theme.root, "rdp-month_grid").expect("web month grid");
    let web_title = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label")
        .as_str();

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    const HIJRI_WEEKDAYS: [&str; 7] = [
        "شنبه",
        "یک\u{200c}شنبه",
        "دوشنبه",
        "سه\u{200c}شنبه",
        "چهارشنبه",
        "پنج\u{200c}شنبه",
        "جمعه",
    ];

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| HIJRI_WEEKDAYS.iter().any(|wd| label.starts_with(wd)))
    });
    assert_eq!(
        web_day_buttons.len(),
        42,
        "expected a 6-week (42-day) grid for calendar-hijri"
    );

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;
        use time::{Date, Month};

        let selected_date = Date::from_calendar_date(2025, Month::June, 12).expect("valid date");
        let month = SolarHijriMonth::from_gregorian(selected_date);

        let month_model: Model<SolarHijriMonth> = cx.app.models_mut().insert(month);
        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut cal = fret_ui_shadcn::CalendarHijri::new(month_model, selected)
            .show_outside_days(true)
            .refine_style(chrome_override);
        if let Some(cell_size) = cell_size {
            cal = cal.cell_size(cell_size);
        }

        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |cx| vec![cal.into_element(cx)],
        )]
    });

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    let next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");

    let prev_bounds = ui.debug_node_bounds(prev.id).expect("prev bounds");
    let next_bounds = ui.debug_node_bounds(next.id).expect("next bounds");
    assert_close_px(
        "calendar-hijri prev x",
        prev_bounds.origin.x,
        web_prev.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri prev y",
        prev_bounds.origin.y,
        web_prev.rect.y,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next x",
        next_bounds.origin.x,
        web_next.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next y",
        next_bounds.origin.y,
        web_next.rect.y,
        3.0,
    );

    let title = find_semantics(&snap, SemanticsRole::Text, Some(web_title))
        .expect("fret calendar-hijri title semantics node");
    let web_title_node = find_first(&theme.root, &|n| n.text.as_deref() == Some(web_title))
        .expect("web calendar-hijri title node");
    let title_bounds = ui.debug_node_bounds(title.id).expect("title bounds");
    // Title text width is font-metrics dependent (Persian shaping), so gate the center position.
    let title_center_x = title_bounds.origin.x.0 + title_bounds.size.width.0 / 2.0;
    let web_title_center_x = web_title_node.rect.x + web_title_node.rect.w / 2.0;
    assert_close_px(
        "calendar-hijri title center x",
        Px(title_center_x),
        web_title_center_x,
        3.0,
    );

    for web_day in web_day_buttons {
        let label = web_day.attrs.get("aria-label").expect("web day aria-label");
        let fret_day = find_semantics(&snap, SemanticsRole::Button, Some(label.as_str()))
            .unwrap_or_else(|| panic!("missing fret hijri day button label={label:?}"));
        let fret_bounds = ui.debug_node_bounds(fret_day.id).expect("fret day bounds");

        assert_close_px(
            "calendar-hijri day w",
            fret_bounds.size.width,
            web_day.rect.w,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day h",
            fret_bounds.size.height,
            web_day.rect.h,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day x",
            fret_bounds.origin.x,
            web_day.rect.x,
            3.0,
        );
        assert_close_px(
            "calendar-hijri day y",
            fret_bounds.origin.y,
            web_day.rect.y,
            3.0,
        );
    }
}

pub(super) fn parse_calendar_title_label(label: &str) -> Option<(time::Month, i32)> {
    let label = label.trim();
    let (month, year) = label.rsplit_once(' ')?;
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year: i32 = year.parse().ok()?;

    let month_lower = month.to_lowercase();
    let month = match (month, month_lower.as_str()) {
        ("January", _) | (_, "january") | (_, "enero") => time::Month::January,
        ("February", _) | (_, "february") | (_, "febrero") => time::Month::February,
        ("March", _) | (_, "march") | (_, "marzo") => time::Month::March,
        ("April", _) | (_, "april") | (_, "abril") => time::Month::April,
        ("May", _) | (_, "may") | (_, "mayo") => time::Month::May,
        ("June", _) | (_, "june") | (_, "junio") => time::Month::June,
        ("July", _) | (_, "july") | (_, "julio") => time::Month::July,
        ("August", _) | (_, "august") | (_, "agosto") => time::Month::August,
        ("September", _) | (_, "september") | (_, "septiembre") | (_, "setiembre") => {
            time::Month::September
        }
        ("October", _) | (_, "october") | (_, "octubre") => time::Month::October,
        ("November", _) | (_, "november") | (_, "noviembre") => time::Month::November,
        ("December", _) | (_, "december") | (_, "diciembre") => time::Month::December,
        _ => return None,
    };
    Some((month, year))
}

pub(super) fn parse_calendar_weekday_label(label: &str) -> Option<time::Weekday> {
    let label = label.trim();
    let lower = label.to_lowercase();
    match (label, lower.as_str()) {
        ("Monday", _) | (_, "monday") | (_, "lunes") => Some(time::Weekday::Monday),
        ("Tuesday", _) | (_, "tuesday") | (_, "martes") => Some(time::Weekday::Tuesday),
        ("Wednesday", _) | (_, "wednesday") | (_, "miércoles") | (_, "miercoles") => {
            Some(time::Weekday::Wednesday)
        }
        ("Thursday", _) | (_, "thursday") | (_, "jueves") => Some(time::Weekday::Thursday),
        ("Friday", _) | (_, "friday") | (_, "viernes") => Some(time::Weekday::Friday),
        ("Saturday", _) | (_, "saturday") | (_, "sábado") | (_, "sabado") => {
            Some(time::Weekday::Saturday)
        }
        ("Sunday", _) | (_, "sunday") | (_, "domingo") => Some(time::Weekday::Sunday),
        _ => None,
    }
}

pub(super) fn parse_calendar_day_aria_label(label: &str) -> Option<(time::Date, bool)> {
    let selected = label.ends_with(", selected");
    let label = label.strip_suffix(", selected").unwrap_or(label);
    let label = label.strip_prefix("Today, ").unwrap_or(label);
    let label = label.strip_prefix("Hoy, ").unwrap_or(label);

    if let Some((prefix, year)) = label.rsplit_once(", ") {
        if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
            let year: i32 = year.parse().ok()?;

            let (_weekday, month_and_day) = prefix.split_once(", ")?;
            let (month, day_with_suffix) = month_and_day.split_once(' ')?;
            let (month, _label_year) = parse_calendar_title_label(&format!("{month} {year}"))?;

            let day_digits: String = day_with_suffix
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if day_digits.is_empty() {
                return None;
            }
            let day: u8 = day_digits.parse().ok()?;

            let date = time::Date::from_calendar_date(year, month, day).ok()?;
            return Some((date, selected));
        }
    }

    // e.g. "lunes, 1 de septiembre de 2025"
    let (weekday, rest) = label.split_once(", ")?;
    let _weekday = parse_calendar_weekday_label(weekday)?;
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 5 || parts[1] != "de" || parts[3] != "de" {
        return None;
    }
    let day: u8 = parts[0].parse().ok()?;
    let (month, year) = parse_calendar_title_label(&format!("{} {}", parts[2], parts[4]))?;
    let date = time::Date::from_calendar_date(year, month, day).ok()?;
    Some((date, selected))
}

fn days_in_month(year: i32, month: time::Month) -> u8 {
    match month {
        time::Month::January => 31,
        time::Month::February => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap { 29 } else { 28 }
        }
        time::Month::March => 31,
        time::Month::April => 30,
        time::Month::May => 31,
        time::Month::June => 30,
        time::Month::July => 31,
        time::Month::August => 31,
        time::Month::September => 30,
        time::Month::October => 31,
        time::Month::November => 30,
        time::Month::December => 31,
    }
}

pub(super) fn parse_calendar_cell_size_px(theme: &WebGoldenTheme) -> Option<Px> {
    let rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root")?;
    let class_name = rdp_root.class_name.as_deref().unwrap_or("");

    fn parse_spacing_value(token: &str, prefix: &str) -> Option<f32> {
        let rest = token.strip_prefix(prefix)?;
        let rest = rest.strip_suffix(")]")?;
        rest.parse::<f32>().ok()
    }

    let mut base: Option<f32> = None;
    let mut md: Option<f32> = None;
    for token in class_name.split_whitespace() {
        if let Some(v) = parse_spacing_value(token, "[--cell-size:--spacing(") {
            base = Some(v);
        }
        if let Some(v) = parse_spacing_value(token, "md:[--cell-size:--spacing(") {
            md = Some(v);
        }
    }

    let viewport_w = theme.viewport.w;
    let md_min_width = fret_ui_kit::declarative::viewport_tailwind::MD.0;
    let spacing = if viewport_w >= md_min_width {
        md.or(base)
    } else {
        base
    }?;

    Some(Px(spacing * 4.0))
}

fn assert_calendar_single_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(
        web_month_grids.len(),
        1,
        "expected a single month grid for {web_name} (multi-month variants are gated separately)"
    );
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();

    let web_is_range_mode = find_first(&theme.root, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let web_selected = web_day_buttons
        .iter()
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str().ends_with(", selected"))
        })
        .copied();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_selected.unwrap_or(web_day_buttons[0]);
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use fret_ui_headless::calendar::DateRangeSelection;

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        match web_selected_dates.as_slice() {
            [] | [_] => {
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);
                let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ if web_is_range_mode => {
                let (min, max) = web_selected_dates.iter().fold(
                    (web_selected_dates[0], web_selected_dates[0]),
                    |(min, max), d| (min.min(*d), max.max(*d)),
                );
                let selected: Model<DateRangeSelection> =
                    cx.app.models_mut().insert(DateRangeSelection {
                        from: Some(min),
                        to: Some(max),
                    });
                let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ => {
                let selected: Model<Vec<time::Date>> =
                    cx.app.models_mut().insert(web_selected_dates.clone());
                let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
        }
    });

    let fret_day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| parse_calendar_day_aria_label(label).is_some())
        })
        .count();
    assert_eq!(
        fret_day_buttons,
        web_day_buttons.len(),
        "expected the same number of calendar day buttons for {web_name}"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        &format!("{web_name} prev button width"),
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} prev button height"),
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some(web_sample_label.as_ref()),
    )
    .expect("fret day semantics node");
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );

    if let Some(web_selected) = web_selected {
        let label = web_selected
            .attrs
            .get("aria-label")
            .expect("web selected day label");
        let fret_selected = find_semantics(&snap, SemanticsRole::Button, Some(label))
            .expect("fret selected day semantics node");
        assert!(
            fret_selected.flags.selected,
            "expected fret selected day to have selected semantics flag for {web_name}"
        );
    }
}

fn assert_calendar_multi_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let mut web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    web_month_grids.sort_by(|a, b| {
        let by_y = a.rect.y.total_cmp(&b.rect.y);
        if !matches!(by_y, std::cmp::Ordering::Equal) {
            return by_y;
        }
        a.rect.x.total_cmp(&b.rect.x)
    });
    assert_eq!(
        web_month_grids.len(),
        2,
        "expected two month grids for {web_name}"
    );

    let month_labels: Vec<(time::Month, i32)> = web_month_grids
        .iter()
        .map(|grid| {
            let label = grid
                .attrs
                .get("aria-label")
                .expect("web month grid aria-label");
            let (m, y) = parse_calendar_title_label(label).expect("web month label (Month YYYY)");
            (m, y)
        })
        .collect();
    let (month_a, year_a) = month_labels[0];
    let (month_b, year_b) = month_labels[1];

    let locale = web_month_grids
        .first()
        .and_then(|grid| grid.attrs.get("aria-label"))
        .and_then(|label| label.chars().next())
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let in_view = |d: time::Date| {
        (d.month() == month_a && d.year() == year_a) || (d.month() == month_b && d.year() == year_b)
    };

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    let web_disable_navigation = web_prev
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true")
        && web_next
            .attrs
            .get("aria-disabled")
            .is_some_and(|v| v == "true");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();

    let web_is_range_mode = find_first(&theme.root, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let web_selected = web_day_buttons
        .iter()
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str().ends_with(", selected"))
        })
        .copied();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();
    let web_has_out_of_view_days = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).map(|(d, _)| d))
        .any(|d| !in_view(d));

    let web_month_bounds =
        if web_disable_navigation && web_show_outside_days && !web_has_out_of_view_days {
            Some(((month_a, year_a), (month_b, year_b)))
        } else {
            None
        };

    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if in_view(date) {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_selected.unwrap_or(web_day_buttons[0]);
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(&theme);

    let chrome_override = {
        let mut chrome = ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use fret_ui_headless::calendar::DateRangeSelection;

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(year_a, month_a));

        match web_selected_dates.as_slice() {
            [] | [_] => {
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);
                let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ if web_is_range_mode => {
                let (min, max) = web_selected_dates.iter().fold(
                    (web_selected_dates[0], web_selected_dates[0]),
                    |(min, max), d| (min.min(*d), max.max(*d)),
                );
                let selected: Model<DateRangeSelection> =
                    cx.app.models_mut().insert(DateRangeSelection {
                        from: Some(min),
                        to: Some(max),
                    });
                let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ => {
                let selected: Model<Vec<time::Date>> =
                    cx.app.models_mut().insert(web_selected_dates.clone());
                let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);

                if web_name == "calendar-03" {
                    calendar = calendar.required(true).max(5);
                }
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }

                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
        }
    });

    let fret_day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| parse_calendar_day_aria_label(label).is_some())
        })
        .count();
    assert_eq!(
        fret_day_buttons,
        web_day_buttons.len(),
        "expected the same number of calendar day buttons for {web_name}"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    let next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");

    assert_close_px(
        &format!("{web_name} prev button width"),
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} prev button height"),
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} next button width"),
        next.bounds.size.width,
        web_next.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} next button height"),
        next.bounds.size.height,
        web_next.rect.h,
        1.0,
    );

    let prev_bounds = ui
        .debug_node_bounds(prev.id)
        .expect("fret prev button node bounds");
    assert_close_px(
        &format!("{web_name} prev x"),
        prev_bounds.origin.x,
        web_prev.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} prev y"),
        prev_bounds.origin.y,
        web_prev.rect.y,
        3.0,
    );

    let next_bounds = ui
        .debug_node_bounds(next.id)
        .expect("fret next button node bounds");
    assert_close_px(
        &format!("{web_name} next x"),
        next_bounds.origin.x,
        web_next.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} next y"),
        next_bounds.origin.y,
        web_next.rect.y,
        3.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some(web_sample_label.as_ref()),
    )
    .expect("fret day semantics node");
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );
}

#[test]
fn web_vs_fret_layout_calendar_variant_geometries_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_calendar_variant_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutCalendarVariantCase> =
        serde_json::from_str(raw).expect("layout calendar variant fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!(
            "layout calendar variant case={} web_name={}",
            case.id, case.web_name
        );
        match case.recipe {
            LayoutCalendarVariantRecipe::SingleMonth => {
                assert_calendar_single_month_variant_geometry_matches_web(&case.web_name);
            }
            LayoutCalendarVariantRecipe::MultiMonth => {
                assert_calendar_multi_month_variant_geometry_matches_web(&case.web_name);
            }
        }
    }
}

#[test]
fn web_vs_fret_layout_calendar_01_background_matches_web() {
    let web = read_web_golden("calendar-01");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;
    let web_bg_css = web_rdp_root
        .computed_style
        .get("backgroundColor")
        .expect("web calendar root backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(&theme);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        if let Some(cell_size) = cell_size {
            calendar = calendar.cell_size(cell_size);
        }
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(Px(web_rdp_root.rect.x), Px(web_rdp_root.rect.y)),
        CoreSize::new(Px(web_rdp_root.rect.w), Px(web_rdp_root.rect.h)),
    );
    let quad = find_best_background_quad(&scene, target).expect("painted calendar background quad");

    assert_rect_xwh_close_px("calendar-01 root quad", quad.rect, web_rdp_root.rect, 3.0);
    assert_rgba_close(
        "calendar-01 root background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_selected_day_background_matches_web() {
    let web = read_web_golden("calendar-14");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    // New shadcn/day-picker versions no longer annotate aria-label with ", selected", and
    // aria-selected can live on a containing gridcell instead of the button. Find a selected cell
    // and then locate its day button.
    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));
    let web_bg_css = web_selected_button
        .computed_style
        .get("backgroundColor")
        .expect("web selected day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    // Some calendar variants don't expose the cell size contract via a CSS variable in the golden.
    // Fall back to the measured web day button width to keep the geometry gate stable.
    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(
            Px(web_selected_button.rect.x),
            Px(web_selected_button.rect.y),
        ),
        CoreSize::new(
            Px(web_selected_button.rect.w),
            Px(web_selected_button.rect.h),
        ),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque selected day background quad");

    assert_rect_xwh_close_px(
        "calendar-14 selected day quad",
        quad.rect,
        web_selected_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14 selected day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_selected_day_background_matches_web() {
    let web = read_web_golden("calendar-14.vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    // New shadcn/day-picker versions no longer annotate aria-label with ", selected", and
    // aria-selected can live on a containing gridcell instead of the button. Find a selected cell
    // and then locate its day button.
    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));
    let web_bg_css = web_selected_button
        .computed_style
        .get("backgroundColor")
        .expect("web selected day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    // Some calendar variants don't expose the cell size contract via a CSS variable in the golden.
    // Fall back to the measured web day button width to keep the geometry gate stable.
    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let target = Rect::new(
        Point::new(
            Px(web_selected_button.rect.x),
            Px(web_selected_button.rect.y),
        ),
        CoreSize::new(
            Px(web_selected_button.rect.w),
            Px(web_selected_button.rect.h),
        ),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque selected day background quad");

    assert_rect_xwh_close_px(
        "calendar-14.vp375x320 selected day quad",
        quad.rect,
        web_selected_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14.vp375x320 selected day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_hover_day_background_matches_web() {
    let web = read_web_golden("calendar-14.hover-day-13");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_hovered_button = web_day_buttons
        .iter()
        .filter(|n| {
            n.computed_style
                .get("backgroundColor")
                .is_some_and(|v| v != "rgba(0, 0, 0, 0)")
        })
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label != web_selected_label)
        })
        .copied()
        .expect("web hovered day button (non-transparent backgroundColor)");
    let web_hovered_label = web_hovered_button
        .attrs
        .get("aria-label")
        .expect("web hovered day aria-label");
    let web_bg_css = web_hovered_button
        .computed_style
        .get("backgroundColor")
        .expect("web hovered day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_hovered_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let hover_button1 = find_semantics(&snap1, SemanticsRole::Button, Some(web_hovered_label))
        .expect("fret hovered day button semantics node (pre-hover)");
    let hover_pos = Point::new(
        Px(hover_button1.bounds.origin.x.0 + hover_button1.bounds.size.width.0 * 0.5),
        Px(hover_button1.bounds.origin.y.0 + hover_button1.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let target = Rect::new(
        Point::new(Px(web_hovered_button.rect.x), Px(web_hovered_button.rect.y)),
        CoreSize::new(Px(web_hovered_button.rect.w), Px(web_hovered_button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque hovered day background quad");

    assert_rect_xwh_close_px(
        "calendar-14 hover day quad",
        quad.rect,
        web_hovered_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14 hover day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_hover_day_background_matches_web() {
    let web = read_web_golden("calendar-14.hover-day-13-vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_hovered_button = web_day_buttons
        .iter()
        .filter(|n| {
            n.computed_style
                .get("backgroundColor")
                .is_some_and(|v| v != "rgba(0, 0, 0, 0)")
        })
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label != web_selected_label)
        })
        .copied()
        .expect("web hovered day button (non-transparent backgroundColor)");
    let web_hovered_label = web_hovered_button
        .attrs
        .get("aria-label")
        .expect("web hovered day aria-label");
    let web_bg_css = web_hovered_button
        .computed_style
        .get("backgroundColor")
        .expect("web hovered day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_hovered_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let render = |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    };

    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");

    let hover_button1 = find_semantics(&snap1, SemanticsRole::Button, Some(web_hovered_label))
        .expect("fret hovered day button semantics node (pre-hover)");
    let hover_pos = Point::new(
        Px(hover_button1.bounds.origin.x.0 + hover_button1.bounds.size.width.0 * 0.5),
        Px(hover_button1.bounds.origin.y.0 + hover_button1.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        render,
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let target = Rect::new(
        Point::new(Px(web_hovered_button.rect.x), Px(web_hovered_button.rect.y)),
        CoreSize::new(Px(web_hovered_button.rect.w), Px(web_hovered_button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque hovered day background quad");

    assert_rect_xwh_close_px(
        "calendar-14.vp375x320 hover day quad",
        quad.rect,
        web_hovered_button.rect,
        3.0,
    );
    assert_rgba_close(
        "calendar-14.vp375x320 hover day background",
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_selected_day_text_rect_matches_web() {
    let web = read_web_golden("calendar-14");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_day_number = {
        let mut stack = vec![web_selected_button];
        let mut best: Option<&WebNode> = None;
        while let Some(node) = stack.pop() {
            for child in &node.children {
                stack.push(child);
            }

            let Some(text) = node.text.as_deref() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() || text.len() > 2 || !text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            best = Some(node);
        }
        best.expect("web selected day number text node")
    };

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, _scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let fret_selected_button =
        find_semantics(&snap, SemanticsRole::Button, Some(web_selected_label))
            .expect("fret selected day button semantics node");

    let fret_day_number_text = {
        let want = web_day_number.text.as_deref().unwrap_or("").trim();
        assert!(!want.is_empty(), "expected web day number text");

        let mut candidates: Vec<&fret_core::SemanticsNode> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Text)
            .filter(|n| n.label.as_deref() == Some(want))
            .filter(|n| {
                let eps = 0.01;
                let outer = fret_selected_button.bounds;
                let inner = n.bounds;
                inner.origin.x.0 + eps >= outer.origin.x.0
                    && inner.origin.y.0 + eps >= outer.origin.y.0
                    && inner.origin.x.0 + inner.size.width.0
                        <= outer.origin.x.0 + outer.size.width.0 + eps
                    && inner.origin.y.0 + inner.size.height.0
                        <= outer.origin.y.0 + outer.size.height.0 + eps
            })
            .collect();

        candidates.sort_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let bw = b.bounds.size.width.0;
            bw.total_cmp(&aw)
        });
        candidates
            .first()
            .copied()
            .unwrap_or_else(|| panic!("missing fret selected day number text node label={want:?}"))
    };

    // The web golden captures element rects, not glyph bounding boxes. Day numbers are typically
    // flex items whose rect spans the full cell. Gate a high-signal invariant we can check today:
    // the day number text in Fret should be centered within the selected day button.
    let fret_button_cx =
        fret_selected_button.bounds.origin.x.0 + fret_selected_button.bounds.size.width.0 / 2.0;
    let fret_button_cy =
        fret_selected_button.bounds.origin.y.0 + fret_selected_button.bounds.size.height.0 / 2.0;
    let fret_text_cx =
        fret_day_number_text.bounds.origin.x.0 + fret_day_number_text.bounds.size.width.0 / 2.0;
    let fret_text_cy =
        fret_day_number_text.bounds.origin.y.0 + fret_day_number_text.bounds.size.height.0 / 2.0;

    assert_close_px(
        "calendar-14 day number center x ~= button center x",
        Px(fret_text_cx),
        fret_button_cx,
        2.5,
    );
    assert_close_px(
        "calendar-14 day number center y ~= button center y",
        Px(fret_text_cy),
        fret_button_cy,
        2.5,
    );
}

#[test]
fn web_vs_fret_layout_calendar_14_vp375x320_selected_day_text_rect_matches_web() {
    let web = read_web_golden("calendar-14.vp375x320");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_day_number = {
        let mut stack = vec![web_selected_button];
        let mut best: Option<&WebNode> = None;
        while let Some(node) = stack.pop() {
            for child in &node.children {
                stack.push(child);
            }

            let Some(text) = node.text.as_deref() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() || text.len() > 2 || !text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            best = Some(node);
        }
        best.expect("web selected day number text node")
    };

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let selected_day_cell_size_px =
        parse_calendar_cell_size_px(&theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (snap, _scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .shadow_sm(),
            );
        calendar = calendar.cell_size(selected_day_cell_size_px);
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let fret_selected_button =
        find_semantics(&snap, SemanticsRole::Button, Some(web_selected_label))
            .expect("fret selected day button semantics node");

    let fret_day_number_text = {
        let want = web_day_number.text.as_deref().unwrap_or("").trim();
        assert!(!want.is_empty(), "expected web day number text");

        let mut candidates: Vec<&fret_core::SemanticsNode> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Text)
            .filter(|n| n.label.as_deref() == Some(want))
            .filter(|n| {
                let eps = 0.01;
                let outer = fret_selected_button.bounds;
                let inner = n.bounds;
                inner.origin.x.0 + eps >= outer.origin.x.0
                    && inner.origin.y.0 + eps >= outer.origin.y.0
                    && inner.origin.x.0 + inner.size.width.0
                        <= outer.origin.x.0 + outer.size.width.0 + eps
                    && inner.origin.y.0 + inner.size.height.0
                        <= outer.origin.y.0 + outer.size.height.0 + eps
            })
            .collect();

        candidates.sort_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let bw = b.bounds.size.width.0;
            bw.total_cmp(&aw)
        });
        candidates
            .first()
            .copied()
            .unwrap_or_else(|| panic!("missing fret selected day number text node label={want:?}"))
    };

    // The web golden captures element rects, not glyph bounding boxes. Day numbers are typically
    // flex items whose rect spans the full cell. Gate a high-signal invariant we can check today:
    // the day number text in Fret should be centered within the selected day button.
    let fret_button_cx =
        fret_selected_button.bounds.origin.x.0 + fret_selected_button.bounds.size.width.0 / 2.0;
    let fret_button_cy =
        fret_selected_button.bounds.origin.y.0 + fret_selected_button.bounds.size.height.0 / 2.0;
    let fret_text_cx =
        fret_day_number_text.bounds.origin.x.0 + fret_day_number_text.bounds.size.width.0 / 2.0;
    let fret_text_cy =
        fret_day_number_text.bounds.origin.y.0 + fret_day_number_text.bounds.size.height.0 / 2.0;

    assert_close_px(
        "calendar-14.vp375x320 day number center x ~= button center x",
        Px(fret_text_cx),
        fret_button_cx,
        2.5,
    );
    assert_close_px(
        "calendar-14.vp375x320 day number center y ~= button center y",
        Px(fret_text_cy),
        fret_button_cy,
        2.5,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_range_middle_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_middle"))
        .expect("web range-middle day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-middle day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-middle day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_middle", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_range_start_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_start"))
        .expect("web range-start day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-start day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-start day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_start", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_range_end_day_background_matches_web() {
    let web = read_web_golden("calendar-04");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_end"))
        .expect("web range-end day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-end day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-end day aria-label");
    assert_calendar_range_day_background_matches_web("calendar-04", "rdp-range_end", label);
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_middle_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_middle"))
        .expect("web range-middle day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-middle day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-middle day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_middle",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_start_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_start"))
        .expect("web range-start day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-start day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-start day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_start",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_04_vp375x320_range_end_day_background_matches_web() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web_theme(&web);
    let cell = find_first(&theme.root, &|n| class_has_token(n, "rdp-range_end"))
        .expect("web range-end day cell");
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .expect("web range-end day button");
    let label = button
        .attrs
        .get("aria-label")
        .expect("web range-end day aria-label");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_end",
        label,
    );
}

#[test]
fn web_vs_fret_layout_calendar_22_open_background_matches_web() {
    let web = read_web_golden("calendar-22.open");
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_bg_css = web_rdp_root
        .computed_style
        .get("backgroundColor")
        .expect("web calendar root backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_month_grids = find_all_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(web_month_grids.len(), 1, "expected a single month grid");
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_weekday_headers = find_all_in_theme(theme, &|n| class_has_token(n, "rdp-weekday"));
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_day_buttons = find_all_in_theme(theme, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for calendar-22.open"
    );
    let web_show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(&theme);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    use fret_ui_headless::calendar::CalendarMonth;
    let open: Model<bool> = app.models_mut().insert(true);
    let month_model: Model<CalendarMonth> =
        app.models_mut().insert(CalendarMonth::new(year, month));
    let selected: Model<Option<time::Date>> = app.models_mut().insert(None);

    let calendar_bg: Rc<Cell<Option<fret_core::Color>>> = Rc::new(Cell::new(None));
    let calendar_bg_for_render = calendar_bg.clone();
    let render = move |cx: &mut fret_ui::ElementContext<'_, App>| {
        use fret_ui_kit::{LengthRefinement, Space};

        let popover =
            fret_ui_shadcn::Popover::new(open.clone()).align(fret_ui_shadcn::PopoverAlign::Start);
        let calendar_bg = calendar_bg_for_render.clone();
        let month_model = month_model.clone();
        let selected = selected.clone();
        vec![popover.into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Select date").into_element(cx),
            move |cx| {
                let mut calendar =
                    fret_ui_shadcn::Calendar::new(month_model.clone(), selected.clone())
                        .week_start(week_start)
                        .show_outside_days(web_show_outside_days)
                        .disable_outside_days(web_disable_outside_days);
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }

                let calendar = calendar.into_element(cx);
                match &calendar.kind {
                    fret_ui::element::ElementKind::Container(props) => {
                        let bg = props
                            .background
                            .expect("calendar root background (resolved)");
                        calendar_bg.set(Some(bg));
                    }
                    other => panic!("expected calendar root container, got {other:?}"),
                }

                fret_ui_shadcn::PopoverContent::new(vec![calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                    .into_element(cx)
            },
        )]
    };

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    for frame in 1..=2 {
        app.set_frame_id(FrameId(frame));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-layout",
            &render,
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    let actual_bg = calendar_bg
        .get()
        .expect("calendar-22.open calendar root background captured");
    assert_rgba_close(
        "calendar-22.open root background",
        color_to_rgba(actual_bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_calendar_background_transparent_in_card_content_scope() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(600.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    use fret_ui_headless::calendar::CalendarMonth;
    let month_model: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, time::Month::January));
    let selected: Model<Option<time::Date>> = app.models_mut().insert(None);

    let calendar_bg: Rc<Cell<Option<fret_core::Color>>> = Rc::new(Cell::new(None));
    let calendar_bg_for_render = calendar_bg.clone();
    let render = move |cx: &mut fret_ui::ElementContext<'_, App>| {
        let calendar_bg = calendar_bg_for_render.clone();
        let month_model = month_model.clone();
        let selected = selected.clone();

        vec![fret_ui_shadcn::card::card_content(cx, move |cx| {
            let calendar = fret_ui_shadcn::Calendar::new(month_model.clone(), selected.clone())
                .into_element(cx);
            match &calendar.kind {
                fret_ui::element::ElementKind::Container(props) => {
                    let bg = props
                        .background
                        .expect("calendar root background (resolved)");
                    calendar_bg.set(Some(bg));
                }
                other => panic!("expected calendar root container, got {other:?}"),
            }

            [calendar]
        })]
    };

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        &render,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let actual_bg = calendar_bg
        .get()
        .expect("calendar card-content background captured");
    assert!(
        color_to_rgba(actual_bg).a <= 0.001,
        "expected transparent calendar bg inside CardContent, got {:?}",
        color_to_rgba(actual_bg)
    );
}
