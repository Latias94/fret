use super::*;

#[path = "support/paint.rs"]
mod paint;
#[path = "support/services.rs"]
mod services;

#[path = "support/geometry.rs"]
mod geometry;
#[path = "support/scene.rs"]
mod scene;
#[path = "support/ui_tree.rs"]
mod ui_tree;

pub(crate) use geometry::*;
pub(crate) use paint::*;
pub(crate) use scene::*;
pub(crate) use services::*;
pub(crate) use ui_tree::*;

pub(super) struct CalendarRangeWebConfig {
    month: time::Month,
    year: i32,
    origin_x: f32,
    origin_y: f32,
    chrome_override: ChromeRefinement,
    cell_size: Px,
    week_start: time::Weekday,
    today: Option<time::Date>,
    show_week_number: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    range_min: time::Date,
    range_max: time::Date,
}

pub(super) fn web_calendar_range_config(theme: &WebGoldenTheme) -> CalendarRangeWebConfig {
    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let padding_left = web_css_px(web_rdp_root, "paddingLeft");
    let border_left = web_css_px(web_rdp_root, "borderLeftWidth");

    let web_month_grid = find_first_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
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

    let selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    assert!(
        selected_dates.len() >= 3,
        "expected at least 3 selected dates for range mode"
    );

    let (range_min, range_max) = selected_dates
        .iter()
        .copied()
        .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
            (min.min(d), max.max(d))
        });

    let weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();

    let disable_outside_days = web_day_buttons.iter().any(|n| {
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

    let cell_size = parse_calendar_cell_size_px(theme).unwrap_or_else(|| {
        let sample = web_day_buttons[0];
        Px(sample.rect.w)
    });

    let mut chrome_override = ChromeRefinement::default();
    if (padding_left.0 - 0.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N0);
    } else if (padding_left.0 - 12.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N3);
    } else if (padding_left.0 - 8.0).abs() < 0.5 {
        chrome_override = chrome_override.p(Space::N2);
    }
    if border_left.0 >= 0.5 {
        chrome_override = chrome_override.border_1();
    }

    CalendarRangeWebConfig {
        month,
        year,
        origin_x,
        origin_y,
        chrome_override,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        range_min,
        range_max,
    }
}

pub(super) fn render_fret_calendar_range_scene(
    config: &CalendarRangeWebConfig,
    viewport: WebViewport,
) -> Scene {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    );

    let (_snap, scene) = render_and_paint_in_bounds(bounds, |cx| {
        use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(config.year, config.month));
        let selected: Model<DateRangeSelection> = cx.app.models_mut().insert(DateRangeSelection {
            from: Some(config.range_min),
            to: Some(config.range_max),
        });

        let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
            .week_start(config.week_start)
            .show_outside_days(config.show_outside_days)
            .disable_outside_days(config.disable_outside_days)
            .show_week_number(config.show_week_number)
            .refine_style(config.chrome_override.clone())
            .cell_size(config.cell_size);

        if let Some(today) = config.today {
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
                    left: Px(config.origin_x),
                    top: Px(config.origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    scene
}

pub(super) fn assert_calendar_range_day_background_matches_web(
    web_name: &str,
    range_cell_class: &str,
    expected_label: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let cell = find_first(&theme.root, &|n| class_has_token(n, range_cell_class))
        .unwrap_or_else(|| panic!("web missing {range_cell_class} day cell"));
    let button = find_first(cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str() == expected_label)
    })
    .unwrap_or_else(|| {
        panic!("web missing {range_cell_class} day button label={expected_label:?}")
    });

    let web_bg_css = button
        .computed_style
        .get("backgroundColor")
        .expect("web day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let config = web_calendar_range_config(theme);
    let scene = render_fret_calendar_range_scene(&config, theme.viewport);

    let target = Rect::new(
        Point::new(Px(button.rect.x), Px(button.rect.y)),
        CoreSize::new(Px(button.rect.w), Px(button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .unwrap_or_else(|| panic!("painted opaque {range_cell_class} day background quad"));

    assert_rect_xwh_close_px(
        &format!("{web_name} {range_cell_class} day quad"),
        quad.rect,
        button.rect,
        3.0,
    );
    assert_rgba_close(
        &format!("{web_name} {range_cell_class} day background"),
        paint_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}
