use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use time::{Date, Month};

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::click_at;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(700.0)),
    )
}

fn center_of(bounds: fret_core::Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id: u64,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    app.set_frame_id(FrameId(frame_id));
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "date-picker-close-on-select",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_frames<F>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    start_frame: u64,
    frame_count: u64,
    mut root: F,
) where
    F: FnMut(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
{
    for offset in 0..frame_count {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            start_frame + offset,
            |cx| root(cx),
        );
    }
}

fn click_test_id(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    test_id: &str,
) {
    let bounds = ui
        .semantics_snapshot()
        .expect("semantics snapshot")
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .map(|node| node.bounds)
        .unwrap_or_else(|| panic!("missing semantics node with test_id={test_id:?}"));
    click_at(ui, app, services, center_of(bounds));
}

fn setup_harness() -> (AppWindowId, Rect, App, UiTree<App>, FakeServices) {
    let window = AppWindowId::default();
    let bounds = test_bounds();
    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let mut ui = UiTree::new();
    ui.set_window(window);
    (window, bounds, app, ui, FakeServices)
}

#[test]
fn date_picker_default_selection_keeps_popover_open() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected: Model<Option<Date>> = app.models_mut().insert(None);
    let expected = Date::from_calendar_date(2026, Month::January, 15).expect("expected date");

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DatePicker::new(open.clone(), month.clone(), selected.clone())
                    .test_id_prefix("date-picker")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-picker-calendar:2026-01-15",
    );

    assert_eq!(app.models().get_cloned(&selected), Some(Some(expected)));
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn date_picker_close_on_select_true_closes_after_selection() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::January));
    let selected: Model<Option<Date>> = app.models_mut().insert(None);
    let expected = Date::from_calendar_date(2026, Month::January, 15).expect("expected date");

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DatePicker::new(open.clone(), month.clone(), selected.clone())
                    .close_on_select(true)
                    .test_id_prefix("date-picker")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-picker-calendar:2026-01-15",
    );

    assert_eq!(app.models().get_cloned(&selected), Some(Some(expected)));
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn date_range_picker_default_completed_range_keeps_popover_open() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let expected_from = Date::from_calendar_date(2026, Month::March, 10).expect("from date");
    let expected_to = Date::from_calendar_date(2026, Month::March, 14).expect("to date");
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::March));
    let selected: Model<DateRangeSelection> = app.models_mut().insert(DateRangeSelection {
        from: Some(expected_from),
        to: None,
    });

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                    .test_id_prefix("date-range")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-range-calendar:2026-03-14",
    );

    assert_eq!(
        app.models().get_cloned(&selected),
        Some(DateRangeSelection {
            from: Some(expected_from),
            to: Some(expected_to),
        })
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn date_range_picker_close_on_select_true_keeps_popover_open_for_incomplete_selection() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::March));
    let selected: Model<DateRangeSelection> =
        app.models_mut().insert(DateRangeSelection::default());

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                    .close_on_select(true)
                    .test_id_prefix("date-range")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-range-calendar:2026-03-10",
    );

    assert_eq!(
        app.models().get_cloned(&selected),
        Some(DateRangeSelection {
            from: Some(Date::from_calendar_date(2026, Month::March, 10).expect("from date")),
            to: None,
        })
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn date_range_picker_close_on_select_true_closes_after_range_completion() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let expected_from = Date::from_calendar_date(2026, Month::March, 10).expect("from date");
    let expected_to = Date::from_calendar_date(2026, Month::March, 14).expect("to date");
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::March));
    let selected: Model<DateRangeSelection> = app.models_mut().insert(DateRangeSelection {
        from: Some(expected_from),
        to: None,
    });

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DateRangePicker::new(open.clone(), month.clone(), selected.clone())
                    .close_on_select(true)
                    .test_id_prefix("date-range")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-range-calendar:2026-03-14",
    );

    assert_eq!(
        app.models().get_cloned(&selected),
        Some(DateRangeSelection {
            from: Some(expected_from),
            to: Some(expected_to),
        })
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn date_picker_with_presets_default_selection_keeps_popover_open() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::March));
    let selected: Model<Option<Date>> = app.models_mut().insert(None);
    let today = Date::from_calendar_date(2026, Month::March, 15).expect("today");
    let expected = Date::from_calendar_date(2026, Month::March, 12).expect("expected date");

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                    .today(today)
                    .test_id_prefix("date-presets")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-presets-calendar:2026-03-12",
    );

    assert_eq!(app.models().get_cloned(&selected), Some(Some(expected)));
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn date_picker_with_presets_close_on_select_true_closes_after_selection() {
    let (window, bounds, mut app, mut ui, mut services) = setup_harness();
    let open: Model<bool> = app.models_mut().insert(true);
    let month: Model<CalendarMonth> = app
        .models_mut()
        .insert(CalendarMonth::new(2026, Month::March));
    let selected: Model<Option<Date>> = app.models_mut().insert(None);
    let today = Date::from_calendar_date(2026, Month::March, 15).expect("today");
    let expected = Date::from_calendar_date(2026, Month::March, 12).expect("expected date");

    render_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1,
        3,
        |cx| {
            vec![
                shadcn::DatePickerWithPresets::new(open.clone(), month.clone(), selected.clone())
                    .today(today)
                    .close_on_select(true)
                    .test_id_prefix("date-presets")
                    .into_element(cx),
            ]
        },
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "date-presets-calendar:2026-03-12",
    );

    assert_eq!(app.models().get_cloned(&selected), Some(Some(expected)));
    assert_eq!(app.models().get_copied(&open), Some(false));
}
