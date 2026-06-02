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

pub(crate) fn run_material3_headless_bottom_sheet_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet,
    };

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked sheet (standard): non-overlay surface.
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
                            let sheet = DockedBottomSheet::new()
                                .variant(DockedBottomSheetVariant::Standard)
                                .test_id("bottom-sheet-docked")
                                .into_element(cx, |cx| {
                                    vec![
                                        cx.text("Docked bottom sheet"),
                                        Button::new("Primary")
                                            .variant(ButtonVariant::Filled)
                                            .test_id("bottom-sheet-docked-primary")
                                            .into_element(cx),
                                        Button::new("Secondary")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-docked-secondary")
                                            .into_element(cx),
                                    ]
                                });

                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked bottom sheet scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked_standard".to_string(),
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

            // Modal sheet (open): overlay surface + scrim.
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
                let open_model = open.clone();

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
                            let sheet = ModalBottomSheet::new(open_model.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("bottom-sheet-modal")
                                .into_element(
                                    cx,
                                    |cx| {
                                        Button::new("Underlay probe")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-underlay-probe")
                                            .into_element(cx)
                                    },
                                    |cx| {
                                        vec![
                                            cx.text("Modal bottom sheet"),
                                            Button::new("Close")
                                                .variant(ButtonVariant::Filled)
                                                .test_id("bottom-sheet-modal-close")
                                                .into_element(cx),
                                        ]
                                    },
                                );
                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal bottom sheet overlay scene to be stable ({label}, {scale})"
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
                &format!("material3-bottom-sheet.{scale}.{label}"),
                "material3_headless_bottom_sheet_suite_goldens_v1",
                &suite,
            );
        }
    }
}
