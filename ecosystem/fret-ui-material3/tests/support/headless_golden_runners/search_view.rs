use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
        settle_material3_overlay_scene_snapshot_v1, write_or_assert_material3_suite_for_test_v1,
    },
    headless_search_cases::load_material3_search_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_search_view_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::SearchView;

    let search_suite = load_material3_search_golden_suite_v1();
    let search_view_results = search_suite.search_view_results();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in search_suite.search_view_cases() {
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
                    Size::new(Px(720.0), Px(520.0)),
                );

                let open_model = app.models_mut().insert(case.open());
                let query = app.models_mut().insert(String::new());
                let presentation = case.presentation();
                let results = search_view_results.clone();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let results = results.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let content = cx.named("search_view_content", |cx| {
                                let mut props = FlexProps::default();
                                props.direction = fret_core::Axis::Vertical;
                                props.gap = fret_ui::element::SpacingLength::Px(Px(8.0));
                                cx.flex(props, |cx| {
                                    results
                                        .iter()
                                        .map(|label| cx.text(label.clone()))
                                        .collect::<Vec<_>>()
                                })
                            });

                            let search_view = SearchView::new(open_model.clone(), query.clone())
                                .placeholder("Search")
                                .a11y_label("Search")
                                .test_id("sv")
                                .presentation(presentation)
                                .into_element(cx, |_cx| vec![content]);

                            let content = cx.named("search_view_root", |cx| {
                                let mut root = FlexProps::default();
                                root.direction = fret_core::Axis::Vertical;
                                root.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                cx.flex(root, |cx| {
                                    vec![
                                        search_view,
                                        cx.text("Underlay probe"),
                                        cx.text("Underlay probe 2"),
                                    ]
                                })
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 search view overlay scene to be stable ({label}, {scale}, {case_name})"
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
                        28,
                        72,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-search-view.{scale}.{label}"),
                "material3_headless_search_view_suite_goldens_v1",
                &suite,
            );
        }
    }
}
