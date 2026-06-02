use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_top_app_bar_suite_goldens_v1() {
    use fret_icons::ids;
    use fret_ui::element::ContainerProps;
    use fret_ui_material3::{TopAppBar, TopAppBarAction, TopAppBarVariant};

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
                Size::new(Px(420.0), Px(220.0)),
            );

            let make_actions = |extra: usize| -> Vec<TopAppBarAction> {
                let mut actions = vec![
                    TopAppBarAction::new(ids::ui::SEARCH)
                        .a11y_label("Search")
                        .test_id("top-app-bar-search"),
                    TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                        .a11y_label("More actions")
                        .test_id("top-app-bar-more"),
                ];
                if extra >= 1 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::SETTINGS)
                            .a11y_label("Settings")
                            .test_id("top-app-bar-settings"),
                    );
                }
                if extra >= 2 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::PLAY)
                            .a11y_label("Play")
                            .test_id("top-app-bar-play"),
                    );
                }
                actions
            };

            let mut snapshot_case =
                |case_label: &'static str,
                 variant: TopAppBarVariant,
                 scrolled: bool,
                 actions: Vec<TopAppBarAction>| {
                    let render = |ui: &mut UiTree<TestHost>,
                                  app: &mut TestHost,
                                  services: &mut dyn UiServices| {
                        let actions = actions.clone();
                        fret_ui::declarative::render_root(
                            ui,
                            app,
                            services,
                            window,
                            bounds,
                            "top_app_bar_root",
                            move |cx| {
                                let theme = Theme::global(&*cx.app).clone();

                                let mut bg = ContainerProps::default();
                                bg.layout.size.width = fret_ui::element::Length::Fill;
                                bg.layout.size.height = fret_ui::element::Length::Fill;
                                bg.background = Some(theme.color_token("md.sys.color.background"));

                                let bar = TopAppBar::new(case_label)
                                    .variant(variant)
                                    .scrolled(scrolled)
                                    .navigation_icon(
                                        TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                                            .a11y_label("Navigate")
                                            .test_id("top-app-bar-nav"),
                                    )
                                    .actions(actions)
                                    .test_id("top-app-bar");

                                vec![cx.container(bg, move |cx| vec![bar.into_element(cx)])]
                            },
                        )
                    };

                    let stable_message = format!(
                        "expected the Material3 top app bar scene to be stable after animations settle ({label}, {scale}, {case_label})"
                    );
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &stable_message,
                        &render,
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert(
                "small.idle".to_string(),
                snapshot_case("Small", TopAppBarVariant::Small, false, make_actions(0)),
            );
            cases.insert(
                "small.scrolled".to_string(),
                snapshot_case(
                    "Small (scrolled)",
                    TopAppBarVariant::Small,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.idle".to_string(),
                snapshot_case(
                    "Small Centered",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.scrolled".to_string(),
                snapshot_case(
                    "Small Centered (scrolled)",
                    TopAppBarVariant::SmallCentered,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.wide_actions".to_string(),
                snapshot_case(
                    "Small Centered (wide actions)",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(2),
                ),
            );
            cases.insert(
                "medium.idle".to_string(),
                snapshot_case("Medium", TopAppBarVariant::Medium, false, make_actions(0)),
            );
            cases.insert(
                "medium.scrolled".to_string(),
                snapshot_case(
                    "Medium (scrolled)",
                    TopAppBarVariant::Medium,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "large.idle".to_string(),
                snapshot_case("Large", TopAppBarVariant::Large, false, make_actions(0)),
            );
            cases.insert(
                "large.scrolled".to_string(),
                snapshot_case(
                    "Large (scrolled)",
                    TopAppBarVariant::Large,
                    true,
                    make_actions(0),
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-top-app-bar.{scale}.{label}"),
                "material3_headless_top_app_bar_suite_goldens_v1",
                &suite,
            );
        }
    }
}
