use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_divider_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, SpacerProps};
    use fret_ui_material3::Divider;

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
                Size::new(Px(300.0), Px(220.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                    let content = cx.flex(props, |cx| {
                        let mut row = FlexProps::default();
                        row.direction = fret_core::Axis::Horizontal;
                        row.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                        row.layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                        row.layout.size.height = fret_ui::element::Length::Px(Px(32.0));

                        vec![
                            Divider::horizontal()
                                .test_id("divider-horizontal")
                                .into_element(cx),
                            cx.flex(row, |cx| {
                                vec![
                                    cx.spacer(SpacerProps::default()),
                                    Divider::vertical()
                                        .test_id("divider-vertical")
                                        .into_element(cx),
                                    cx.spacer(SpacerProps::default()),
                                ]
                            }),
                        ]
                    });

                    vec![with_padding(cx, Px(24.0), content)]
                })
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 divider scene to be stable after animations settle ({label}, {scale})"
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
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-divider.{scale}.{label}"),
                "material3_headless_divider_suite_goldens_v1",
                &suite,
            );
        }
    }
}
