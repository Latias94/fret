use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_interactions::{
        dispatch_idle_pointer, dispatch_key_tap, focus_test_id, hover_test_id,
    },
    headless_text_field_cases::load_material3_text_field_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_text_field_suite_goldens_v1() {
    use fret_ui::element::FlexProps;

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
    let text_field_suite = load_material3_text_field_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in text_field_suite.cases() {
                let case_name = case.id();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(640.0), Px(520.0)),
                );

                let field_models = text_field_suite
                    .fields()
                    .iter()
                    .map(|field| {
                        let model = app.models_mut().insert(field.value().to_string());
                        (field, model)
                    })
                    .collect::<Vec<_>>();

                let render = |ui: &mut UiTree<TestHost>,
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
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(12.0));
                            let content = cx.flex(props, |cx| {
                                field_models
                                    .iter()
                                    .map(|(field, model)| {
                                        field.text_field(model.clone()).into_element(cx)
                                    })
                                    .collect::<Vec<_>>()
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                if case.is_idle() {
                    dispatch_idle_pointer(&mut ui, &mut app, &mut services);
                }

                if let Some(test_id) = case.hover_test_id() {
                    hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);
                    dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
                }

                let message = format!(
                    "expected text field scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                let snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    case.settle_from_frame(),
                    case.total_frames(),
                    &message,
                    &render,
                );
                cases.insert(case.id().to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-text-field.{scale}.{label}"),
                "material3_headless_text_field_suite_goldens_v1",
                &suite,
            );
        }
    }
}
