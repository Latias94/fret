use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{UiTree, element::AnyElement, elements::ElementContext};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{IconButton, IconButtonVariant, IconToggleButton};

use super::scale_segment;
use crate::support::{
    events::pointer_move,
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_icon_button_cases::{
        Material3IconButtonGoldenCaseKindV1, load_material3_icon_button_golden_suite_v1,
    },
    headless_interactions::{dispatch_key_tap, focus_test_id, hover_test_id},
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_icon_button_suite_goldens_v1() {
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
    let icon_button_suite = load_material3_icon_button_golden_suite_v1();

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(320.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in icon_button_suite.cases() {
                let case_name = case.id();
                let context = format!("{label}, {scale}, {case_name}");

                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let standard_checked = app.models_mut().insert(true);
                let filled_checked = app.models_mut().insert(true);
                let tonal_checked = app.models_mut().insert(true);
                let outlined_checked = app.models_mut().insert(true);

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "icon_button_root",
                        |cx| {
                            let content = icon_button_matrix(
                                cx,
                                standard_checked.clone(),
                                filled_checked.clone(),
                                tonal_checked.clone(),
                                outlined_checked.clone(),
                            );
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                ui.set_focus(None);
                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                );

                match case.kind() {
                    Material3IconButtonGoldenCaseKindV1::Idle => {}
                    Material3IconButtonGoldenCaseKindV1::Hover => {
                        let _ = hover_test_id(
                            &mut ui,
                            &mut app,
                            &mut services,
                            case.target_test_id(),
                            &context,
                        );
                    }
                    Material3IconButtonGoldenCaseKindV1::FocusVisible => {
                        focus_test_id(&mut ui, case.target_test_id(), &context);
                        dispatch_key_tap(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
                    }
                }

                let message = format!(
                    "expected the Material3 icon button {case_name} scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-icon-button.{scale}.{label}"),
                "material3_headless_icon_button_suite_goldens_v1",
                &suite,
            );
        }
    }
}

fn icon_button_matrix(
    cx: &mut ElementContext<'_, TestHost>,
    standard_checked: Model<bool>,
    filled_checked: Model<bool>,
    tonal_checked: Model<bool>,
    outlined_checked: Model<bool>,
) -> AnyElement {
    let mut column = fret_ui::element::FlexProps::default();
    column.direction = fret_core::Axis::Vertical;
    column.gap = fret_ui::element::SpacingLength::Px(Px(16.0));

    cx.flex(column, |cx| {
        vec![
            icon_button_variant_row(
                cx,
                IconButtonVariant::Standard,
                "icon-standard",
                standard_checked,
            ),
            icon_button_variant_row(cx, IconButtonVariant::Filled, "icon-filled", filled_checked),
            icon_button_variant_row(cx, IconButtonVariant::Tonal, "icon-tonal", tonal_checked),
            icon_button_variant_row(
                cx,
                IconButtonVariant::Outlined,
                "icon-outlined",
                outlined_checked,
            ),
        ]
    })
}

fn icon_button_variant_row(
    cx: &mut ElementContext<'_, TestHost>,
    variant: IconButtonVariant,
    prefix: &'static str,
    checked: Model<bool>,
) -> AnyElement {
    let mut row = fret_ui::element::FlexProps::default();
    row.direction = fret_core::Axis::Horizontal;
    row.gap = fret_ui::element::SpacingLength::Px(Px(12.0));

    cx.flex(row, move |cx| {
        vec![
            IconButton::new(fret_icons::ids::ui::SEARCH)
                .variant(variant)
                .a11y_label("icon button")
                .test_id(format!("{prefix}-button"))
                .into_element(cx),
            IconButton::new(fret_icons::ids::ui::SETTINGS)
                .variant(variant)
                .disabled(true)
                .a11y_label("disabled icon button")
                .test_id(format!("{prefix}-disabled"))
                .into_element(cx),
            IconButton::new(fret_icons::ids::ui::CHECK)
                .variant(variant)
                .toggle(true)
                .selected(false)
                .a11y_label("unselected icon toggle")
                .test_id(format!("{prefix}-toggle-unselected"))
                .into_element(cx),
            IconButton::new(fret_icons::ids::ui::CHECK)
                .variant(variant)
                .toggle(true)
                .selected(true)
                .a11y_label("selected icon toggle")
                .test_id(format!("{prefix}-toggle-selected"))
                .into_element(cx),
            IconToggleButton::new(checked, fret_icons::ids::ui::CHECK)
                .variant(variant)
                .a11y_label("model icon toggle")
                .test_id(format!("{prefix}-model-toggle"))
                .into_element(cx),
        ]
    })
}
