use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        snapshot_material3_scene_at_frame_v1, write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_progress_indicator_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{CircularProgressIndicator, LinearProgressIndicator};

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

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(260.0)),
            );

            let progress_0 = app.models_mut().insert(0.0f32);
            let progress_30 = app.models_mut().insert(0.3f32);
            let progress_100 = app.models_mut().insert(1.0f32);

            let render_determinate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::new(progress_0.clone())
                                        .test_id("linear-0")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_30.clone())
                                        .test_id("linear-30")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_100.clone())
                                        .test_id("linear-100")
                                        .into_element(cx),
                                    {
                                        let mut row = FlexProps::default();
                                        row.direction = fret_core::Axis::Horizontal;
                                        row.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                        cx.flex(row, |cx| {
                                            vec![
                                                CircularProgressIndicator::new(progress_0.clone())
                                                    .test_id("circular-0")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(progress_30.clone())
                                                    .test_id("circular-30")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(
                                                    progress_100.clone(),
                                                )
                                                .test_id("circular-100")
                                                .into_element(cx),
                                            ]
                                        })
                                    },
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .test_id("linear-indeterminate")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .test_id("circular-indeterminate")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate_four_color =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("linear-indeterminate-four-color")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("circular-indeterminate-four-color")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 progress indicator scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    12,
                    24,
                    &idle_message,
                    &render_determinate,
                ),
            );

            cases.insert(
                "indeterminate.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate,
                ),
            );

            cases.insert(
                "indeterminate.four_color.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate_four_color,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-progress-indicator.{scale}.{label}"),
                "material3_headless_progress_indicator_suite_goldens_v1",
                &suite,
            );
        }
    }
}
