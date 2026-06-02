use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1,
        settle_material3_overlay_scene_snapshot_v1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_menu_dialog_style_cases::{
        Material3MenuDialogStyleGoldenCaseKindV1, load_material3_menu_dialog_style_golden_suite_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_menu_dialog_style_suite_goldens_v1() {
    use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, Length, MainAlign};
    use fret_ui_material3::menu::Menu;
    use fret_ui_material3::{Button, Dialog};

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
    let style_suite = load_material3_menu_dialog_style_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Menu: default vs override (in the same scene).
            {
                let menu_case = style_suite
                    .case(Material3MenuDialogStyleGoldenCaseKindV1::MenuDefaultVsOverride);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let style = menu_case.menu_style();
                let entries = style_suite.menu_entries();

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
                            let default_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("default menu")
                                .test_id("menu-default")
                                .into_element(cx);

                            let override_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("override menu")
                                .test_id("menu-override")
                                .style(style.clone())
                                .into_element(cx);

                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Horizontal;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(32.0));
                            props.align = CrossAlign::Start;
                            props.justify = MainAlign::Center;

                            let content = cx.flex(props, |cx| {
                                let mut left = ContainerProps::default();
                                left.layout.size.width = Length::Px(Px(360.0));
                                let left = cx.container(left, |_cx| vec![default_menu]);

                                let mut right = ContainerProps::default();
                                right.layout.size.width = Length::Px(Px(360.0));
                                let right = cx.container(right, |_cx| vec![override_menu]);

                                vec![left, right]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 menu style scene to be stable ({label}, {scale}, {})",
                    menu_case.id()
                );
                cases.insert(
                    menu_case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        menu_case.settle_from_frame(),
                        menu_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: default open state (modal overlay).
            {
                let dialog_case =
                    style_suite.case(Material3MenuDialogStyleGoldenCaseKindV1::DialogDefault);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);
                let headline = dialog_case.headline().to_string();
                let supporting_text = dialog_case.supporting_text().to_string();
                let actions = dialog_case.dialog_actions();

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
                            let dialog = Dialog::new(open.clone())
                                .headline(headline.clone())
                                .supporting_text(supporting_text.clone())
                                .actions(actions.clone())
                                .test_id("dialog-default")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog default scene to be stable after animations settle ({label}, {scale}, {})",
                    dialog_case.id()
                );
                cases.insert(
                    dialog_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        dialog_case.settle_from_frame(),
                        dialog_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: override surface + text colors.
            {
                let dialog_case =
                    style_suite.case(Material3MenuDialogStyleGoldenCaseKindV1::DialogOverride);
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);
                let headline = dialog_case.headline().to_string();
                let supporting_text = dialog_case.supporting_text().to_string();
                let actions = dialog_case.dialog_actions();
                let style = dialog_case.dialog_style();

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
                            let dialog = Dialog::new(open.clone())
                                .headline(headline.clone())
                                .supporting_text(supporting_text.clone())
                                .actions(actions.clone())
                                .style(style.clone())
                                .test_id("dialog-override")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog override scene to be stable after animations settle ({label}, {scale}, {})",
                    dialog_case.id()
                );
                cases.insert(
                    dialog_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        dialog_case.settle_from_frame(),
                        dialog_case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-menu-dialog-style.{scale}.{label}"),
                "material3_headless_menu_dialog_style_suite_goldens_v1",
                &suite,
            );
        }
    }
}
