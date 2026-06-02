use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, NodeId, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    events::{key_down, key_up, pointer_down, pointer_move},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_search_cases::load_material3_search_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_search_bar_suite_goldens_v1() {
    use fret_icons::ids::ui;
    use fret_ui_material3::SearchBar;

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
    let search_suite = load_material3_search_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in search_suite.search_bar_cases() {
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
                    Size::new(Px(640.0), Px(240.0)),
                );

                let model = app.models_mut().insert(String::new());
                let model_for_render = model.clone();

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
                            let search_bar = SearchBar::new(model_for_render.clone())
                                .placeholder("Search")
                                .leading_icon(ui::SEARCH)
                                .trailing_icon(ui::CLOSE)
                                .test_id("sb")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), search_bar)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let node_id: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some("sb")).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!("expected sb in semantics snapshot ({label}, {scale}, {case_name})")
                    });
                let node_bounds = ui.debug_node_visual_bounds(node_id).unwrap_or_else(|| {
                    panic!("expected sb bounds ({label}, {scale}, {case_name})")
                });
                let center = Point::new(
                    Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.5),
                    Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                );

                if case_name == "idle" {
                    ui.dispatch_event(
                        &mut app,
                        &mut services,
                        &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                    );
                }

                if case.hover() {
                    ui.dispatch_event(&mut app, &mut services, &pointer_move(PointerId(1), center));
                }

                if case.pressed() {
                    ui.dispatch_event(&mut app, &mut services, &pointer_down(PointerId(1), center));
                }

                if case.focus_visible() {
                    ui.set_focus(Some(node_id));
                    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                }

                let message = format!(
                    "expected the Material3 search bar scene to be stable ({label}, {scale}, {case_name})"
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
                &format!("material3-search-bar.{scale}.{label}"),
                "material3_headless_search_bar_suite_goldens_v1",
                &suite,
            );
        }
    }
}
