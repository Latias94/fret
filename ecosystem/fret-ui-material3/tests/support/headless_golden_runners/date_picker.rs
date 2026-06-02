use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
        settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_date_picker_suite_goldens_v1() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked picker: non-overlay surface.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let picker = DockedDatePicker::new(month.clone(), selected.clone())
                                .variant(DatePickerVariant::Docked)
                                .today(Some(today))
                                .test_id("date-picker-docked")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked date picker scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: overlay + scrim + focus trap.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(860.0), Px(520.0)),
                );

                let open = app.models_mut().insert(true);
                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let dialog = DatePickerDialog::new(
                                open.clone(),
                                month.clone(),
                                selected.clone(),
                            )
                            .today(Some(today))
                            .open_duration_ms(Some(1))
                            .close_duration_ms(Some(1))
                            .test_id("date-picker-modal")
                            .into_element(cx, |cx| {
                                Button::new("Underlay probe")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("date-picker-underlay-probe")
                                    .into_element(cx)
                            });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 date picker modal overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-date-picker.{scale}.{label}"),
                "material3_headless_date_picker_suite_goldens_v1",
                &suite,
            );
        }
    }
}
