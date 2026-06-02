use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
        settle_material3_overlay_scene_snapshot_v1, write_or_assert_material3_suite_for_test_v1,
    },
    headless_snackbar_cases::load_material3_snackbar_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_snackbar_suite_goldens_v1() {
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{SnackbarController, SnackbarHost};

    let snackbar_suite = load_material3_snackbar_golden_suite_v1();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let snapshot_case =
                |case: &support::headless_snackbar_cases::Material3SnackbarGoldenCaseV1| {
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

                    let store = app.models_mut().insert(ToastStore::default());
                    let controller = SnackbarController::new(store.clone());
                    {
                        let mut action_host =
                            fret_ui::action::UiActionHostAdapter { app: &mut app };
                        let _id = controller.show(&mut action_host, window, case.to_snackbar());
                    }

                    let render =
                        move |ui: &mut UiTree<TestHost>,
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
                                    let host_layer = SnackbarHost::new(store.clone())
                                        .max_snackbars(1)
                                        .into_element(cx);

                                    vec![with_padding(cx, Px(24.0), host_layer)]
                                },
                            )
                        };

                    let message = format!(
                        "expected the Material3 snackbar overlay scene to be stable ({label}, {scale}, {})",
                        case.id()
                    );
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &message,
                        &render,
                    )
                };

            for case in snackbar_suite.cases() {
                cases.insert(case.id().to_string(), snapshot_case(case));
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-snackbar.{scale}.{label}"),
                "material3_headless_snackbar_suite_goldens_v1",
                &suite,
            );
        }
    }
}
