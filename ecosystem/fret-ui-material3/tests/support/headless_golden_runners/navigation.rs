use std::{collections::BTreeMap, sync::Arc};

use fret_core::{AppWindowId, Px, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Button, ButtonVariant, ModalNavigationDrawer, NavigationDrawerVariant};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
        settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_navigation_cases::load_material3_navigation_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_navigation_suite_goldens_v1() {
    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];
    let navigation_suite = load_material3_navigation_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            {
                let case = navigation_suite.case("bar.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let bar = case.navigation_bar(value.clone()).into_element(cx);

                            vec![with_padding(cx, Px(24.0), bar)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation bar scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            {
                let case = navigation_suite.case("rail.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let rail = case.navigation_rail(value.clone()).into_element(cx);

                            vec![with_padding(cx, Px(24.0), rail)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation rail scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            {
                let case = navigation_suite.case("drawer.selected");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

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
                            let drawer = case
                                .navigation_drawer(value.clone(), NavigationDrawerVariant::Standard)
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), drawer)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation drawer scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            {
                let case = navigation_suite.case("modal_drawer.open");
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = case.bounds();

                let open = app.models_mut().insert(true);
                let value: Model<Arc<str>> = app.models_mut().insert(case.selected_value());

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let value = value.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let panel_value = value.clone();
                            let panel =
                                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                                    case.navigation_drawer(
                                        panel_value.clone(),
                                        NavigationDrawerVariant::Modal,
                                    )
                                    .into_element(cx)
                                };

                            let underlay =
                                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                                    Button::new(case.underlay_label())
                                        .variant(ButtonVariant::Outlined)
                                        .test_id(case.underlay_test_id())
                                        .into_element(cx)
                                };

                            let modal = ModalNavigationDrawer::new(open.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id(case.modal_test_id())
                                .into_element(cx, panel, underlay);

                            vec![with_padding(cx, Px(24.0), modal)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal navigation drawer overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-navigation.{scale}.{label}"),
                "material3_headless_navigation_suite_goldens_v1",
                &suite,
            );
        }
    }
}
