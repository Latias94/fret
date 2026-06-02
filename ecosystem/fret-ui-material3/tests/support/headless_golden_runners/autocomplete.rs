use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, NodeId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::{UiTree, element::ContainerProps};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    events::{key_down, key_up},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, run_overlay_frame_scaled,
        settle_material3_overlay_scene_snapshot_v1, write_or_assert_material3_suite_for_test_v1,
    },
    headless_autocomplete_cases::load_material3_autocomplete_golden_suite_v1,
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_autocomplete_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteVariant};

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
    let autocomplete_suite = load_material3_autocomplete_golden_suite_v1();
    let autocomplete_items = autocomplete_suite.items();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let closed_case = autocomplete_suite.closed_case();

            // Closed scene: show both variants so token drift is visible.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items = autocomplete_items.clone();

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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 autocomplete closed scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    closed_case.id().to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        16,
                        32,
                        &message,
                        &render,
                    ),
                );
            }

            for case in autocomplete_suite.open_cases() {
                let case_name = case.id();
                let focus_test_id = case.focus_test_id();

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items = autocomplete_items.clone();

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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let input_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some(focus_test_id)).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "expected {focus_test_id} input node in semantics snapshot ({label}, {scale}, {case_name})"
                        )
                    });

                ui.set_focus(Some(input_node));
                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
                ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                assert!(
                    stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    }),
                    "expected autocomplete popover overlay to be open after ArrowDown ({label}, {scale}, {case_name})"
                );

                let message = format!(
                    "expected the Material3 autocomplete overlay scene to be stable after animations settle ({label}, {scale}, {case_name})"
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
                        44,
                        80,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-autocomplete.{scale}.{label}"),
                "material3_headless_autocomplete_suite_goldens_v1",
                &suite,
            );
        }
    }
}
